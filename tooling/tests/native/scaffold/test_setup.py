import hashlib
import json
import os
import subprocess
import tomllib
from pathlib import Path

import pytest
import yaml
from support import file_contents, invoke, update_config


def declaration(worker: Path, root: Path, service: str = ".agentrig") -> Path:
    seed = root / "seed"
    seed.mkdir()
    result = invoke(worker, seed, "init", "--review", "true", "--service", service)
    assert result.returncode == 0, result.stderr
    consumer = root / "consumer"
    consumer.mkdir()
    (consumer / "agentrig.yaml").write_text(
        "# Consumer policy\n" + (seed / "agentrig.yaml").read_text()
    )
    (consumer / "src").mkdir()
    (consumer / "src/value.py").write_text("value = 1\n")
    return consumer


@pytest.mark.parametrize("service", [".agentrig", "rig space's $cash"])
def test_setup_prepares_and_repeats_without_losing_settings(
    worker: Path, tmp_path: Path, service: str
) -> None:
    root = declaration(worker, tmp_path, service)
    (root / ".codex").mkdir()
    settings = "# Keep this comment\nmodel = 'consumer-model'\n[features]\nhooks = true # enabled\n"
    (root / ".codex/config.toml").write_text(settings)
    (root / ".codex/config.toml").chmod(0o600)
    result = invoke(worker, root, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    actual = (root / ".codex/config.toml").read_text()
    assert actual.startswith(settings)
    assert (root / ".codex/config.toml").stat().st_mode & 0o777 == 0o600
    assert (root / "agentrig.yaml").read_text().startswith("# Consumer policy\n")
    config = tomllib.loads(actual)["mcp_servers"]["worker_review"]
    messages = [
        dict(jsonrpc="2.0", id=1, method="initialize", params=dict(protocolVersion="2025-11-25")),
        dict(jsonrpc="2.0", id=2, method="tools/list"),
    ]
    connected = subprocess.run(
        [config["command"], *config["args"]],
        cwd=root,
        input="".join(json.dumps(message) + "\n" for message in messages),
        capture_output=True,
        text=True,
        check=False,
    )
    assert connected.returncode == 0, connected.stderr
    assert len(json.loads(connected.stdout.splitlines()[1])["result"]["tools"]) == 2
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    assert file_contents(root) == before


def test_setup_preserves_changed_review_and_memory(worker: Path, tmp_path: Path) -> None:
    root = declaration(worker, tmp_path)
    assert invoke(worker, root, "setup").returncode == 0
    review = root / ".agentrig/review/config/review.yaml"
    review.write_text(review.read_text().replace("model: gpt-5.6-luna", "model: consumer-model"))
    state = root / "memory/State.md"
    state.write_text(state.read_text().replace("Not recorded.", "Consumer-owned state."))
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    assert file_contents(root) == before


def test_setup_rejects_conflicting_assets_without_writing(worker: Path, tmp_path: Path) -> None:
    root = declaration(worker, tmp_path)
    assert invoke(worker, root, "setup").returncode == 0
    skill = root / ".agentrig/skills/repair/SKILL.md"
    skill.write_text(skill.read_text() + "\nConsumer instruction.\n")
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 2
    assert "setup conflict" in result.stderr
    assert file_contents(root) == before


@pytest.mark.parametrize("settings", ["features=false\n", "[features]\nhooks=false\n"])
def test_setup_rejects_hook_conflicts_before_writing(
    worker: Path, tmp_path: Path, settings: str
) -> None:
    root = declaration(worker, tmp_path)
    (root / ".codex").mkdir()
    (root / ".codex/config.toml").write_text(settings)
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 2
    assert "setup conflict" in result.stderr
    assert file_contents(root) == before


def test_setup_rejects_unknown_capability_before_writing(worker: Path, tmp_path: Path) -> None:
    root = declaration(worker, tmp_path)
    config = root / "agentrig.yaml"
    config.write_text(
        config.read_text().replace("\ncapabilities:\n", "\ncapabilities:\n  unknown: true\n")
    )
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 2
    assert "unknown field" in result.stderr
    assert file_contents(root) == before


def test_setup_reports_required_scope_backend_failure(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    root = declaration(worker, tmp_path)
    config = root / "agentrig.yaml"
    config.write_text(
        config.read_text().replace("foreground: process-group", "foreground: systemd")
    )
    launcher = tmp_path / "systemd-run"
    launcher.write_text("#!/bin/sh\necho 'scope unavailable' >&2\nexit 1\n")
    launcher.chmod(0o755)
    monkeypatch.setenv("PATH", str(tmp_path) + os.pathsep + os.environ["PATH"])
    result = invoke(worker, root, "setup")
    assert result.returncode != 0
    assert '"available":false' in result.stdout
    assert "process scope capability" in result.stdout


def delegated_project(worker: Path, root: Path, service: str = ".agentrig") -> Path:
    consumer = declaration(worker, root, service)
    config = consumer / "agentrig.yaml"
    update_config(config, capabilities={"delegation": {"config": "agents/profiles.yaml"}})
    agents = consumer / "agents"
    agents.mkdir()
    (agents / "prompt.md").write_text("Read the task and return its required JSON result.\n")
    (agents / "profiles.yaml").write_text(
        "schema_version: 1\nprofiles:\n  reader:\n    frontend: codex\n    model: consumer-model\n"
        "    reasoning_effort: high\n    mode: read\n    prompt: prompt.md\n"
        "    visible_paths: ['src/**']\n    timeout_seconds: 30\n"
        "    credentials:\n      env:\n        OPENAI_API_KEY: CONSUMER_KEY\n"
    )
    return consumer


@pytest.mark.parametrize("service", [".agentrig", "rig space's $cash"])
def test_setup_registers_configured_delegation_and_preserves_resources(
    worker: Path, tmp_path: Path, service: str
) -> None:
    root = delegated_project(worker, tmp_path, service)
    result = invoke(worker, root, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    settings = tomllib.loads((root / ".codex/config.toml").read_text())
    server = settings["mcp_servers"]["worker_delegation"]
    assert server["env_vars"] == [
        "CODEX_SESSION_ID",
        "CODEX_THREAD_ID",
        "CONSUMER_KEY",
        "DELEGATE_CODEX_BIN",
        "WORKER_OWNER",
        "WORKER_PARENT_RUN",
    ]
    messages = [
        dict(jsonrpc="2.0", id=1, method="initialize"),
        dict(jsonrpc="2.0", id=2, method="tools/list"),
    ]
    connected = subprocess.run(
        [server["command"], *server["args"]],
        cwd=root,
        input="".join(json.dumps(message) + "\n" for message in messages),
        capture_output=True,
        text=True,
        check=False,
    )
    assert connected.returncode == 0, connected.stderr
    tools = json.loads(connected.stdout.splitlines()[1])["result"]["tools"]
    assert tools[0]["inputSchema"]["properties"]["profile"]["enum"] == ["reader"]
    installed = root / service / "bin/agentrig"
    assert invoke(installed, root, "delegate", "config-check").returncode == 0
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    assert file_contents(root) == before


def test_setup_rejects_invalid_delegation_before_writing(worker: Path, tmp_path: Path) -> None:
    root = delegated_project(worker, tmp_path)
    (root / "agents/prompt.md").unlink()
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 2
    assert file_contents(root) == before


def test_setup_preserves_conflicting_delegate_registration(worker: Path, tmp_path: Path) -> None:
    root = delegated_project(worker, tmp_path)
    (root / ".codex").mkdir()
    (root / ".codex/config.toml").write_text('[mcp_servers.worker_delegation]\ncommand="custom"\n')
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 2
    assert "worker_delegation.command" in result.stderr
    assert file_contents(root) == before


def test_setup_installs_selected_delegation_skill(worker: Path, tmp_path: Path) -> None:
    root = delegated_project(worker, tmp_path)
    assert not (tmp_path / "seed/.agentrig/skills/delegate-task").exists()
    result = invoke(worker, root, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    path = ".agentrig/skills/delegate-task/SKILL.md"
    assert (root / path).is_file()
    receipt = json.loads((root / ".agentrig/manifest.json").read_text())
    assert receipt["files"][path]["ownership"] == "editable"


def test_composition_cli_previews_reusable_commands_without_installing(
    worker: Path, tmp_path: Path
) -> None:
    root = tmp_path / "consumer"
    root.mkdir()
    assert invoke(worker, root, "init").returncode == 0
    package = tmp_path / "package.yaml"
    package.write_text(
        "schema_version: 1\nid: commands\nversion: '1'\nconfiguration:\n"
        "  commands:\n    hello:\n      argv: [python3, -c, \"print('package command')\"]\n"
    )
    source = root / "declaration.yaml"
    source.write_text(
        (root / "agentrig.yaml").read_text() + "\npackages:\n- path: ../package.yaml\n"
    )
    before = file_contents(tmp_path)
    preview = invoke(worker, root, "config-resolve", str(source))
    assert preview.returncode == 0, preview.stderr
    resolved = json.loads(preview.stdout)
    assert file_contents(tmp_path) == before
    assert resolved["packages"][0]["id"] == "commands"
    assert resolved["provenance"]["/commands/hello/argv"] == str(package)
    assert "packages" not in resolved["configuration"]
    (root / "agentrig.yaml").write_text(yaml.safe_dump(resolved["configuration"], sort_keys=False))
    assert invoke(worker, root, "config-check").returncode == 0
    executed = invoke(worker, root, "run", "hello")
    assert executed.returncode == 0, executed.stderr
    assert "package command" in executed.stdout


def test_setup_preview_matches_application_without_changing_the_consumer(
    worker: Path, tmp_path: Path
) -> None:
    root = declaration(worker, tmp_path, "preview rig")
    before = file_contents(root)
    result = invoke(worker, root, "setup", "--preview")
    assert result.returncode == 0, result.stderr
    preview = json.loads(result.stdout)
    assert file_contents(root) == before
    assert not (root / ".git").exists()
    assert preview["registrations"]["git"]["hooks_path"] == "preview rig/hooks"
    assert preview["registrations"]["git"]["initialize"] is True
    assert "worker_review" in preview["registrations"]["codex"]["mcp_servers"]
    assert set(preview["dependencies"]["executables"]) == {"git", "bwrap", "python3"}
    assert preview["dependencies"]["model_frontends"][0]["override_env"] == "REVIEW_CODEX_BIN"
    applied = invoke(worker, root, "setup")
    assert applied.returncode == 0, applied.stdout + applied.stderr
    for change in preview["files"]:
        path = root / change["path"]
        assert hashlib.sha256(path.read_bytes()).hexdigest() == change["after_sha256"]
        assert path.stat().st_mode & 0o777 == change["mode"]
    for path in preview["directories"]:
        assert (root / path).is_dir()
    repeated = invoke(worker, root, "setup", "--preview")
    assert repeated.returncode == 0, repeated.stderr
    assert json.loads(repeated.stdout)["files"] == []
    just = subprocess.run(
        ["just", "setup", "--preview"], cwd=root, capture_output=True, text=True, check=False
    )
    assert just.returncode == 0, just.stderr
    assert json.loads(just.stdout)["preview"] is True


def test_setup_preview_preserves_conflicts_and_omits_unrelated_settings(
    worker: Path, tmp_path: Path
) -> None:
    root = delegated_project(worker, tmp_path)
    settings = root / ".codex/config.toml"
    settings.parent.mkdir()
    settings.write_text('[mcp_servers.worker_review.env]\nAPI_KEY="consumer-secret"\n')
    before = file_contents(root)
    result = invoke(worker, root, "setup", "--preview")
    assert result.returncode == 0, result.stderr
    preview = json.loads(result.stdout)
    assert "consumer-secret" not in result.stdout
    assert "worker_delegation" in preview["registrations"]["codex"]["mcp_servers"]
    assert preview["dependencies"]["systemd_user_scope"] is True
    assert file_contents(root) == before
    settings.write_text("[features]\nhooks=false\n")
    before = file_contents(root)
    refused = invoke(worker, root, "setup", "--preview")
    assert refused.returncode == 2
    assert "setup conflict" in refused.stderr
    assert file_contents(root) == before
