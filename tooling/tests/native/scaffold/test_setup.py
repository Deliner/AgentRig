import json
import os
import subprocess
import tomllib
from pathlib import Path

import pytest
from support import file_contents, invoke


def declaration(worker: Path, root: Path) -> Path:
    seed = root / "seed"
    seed.mkdir()
    result = invoke(worker, seed, "init", "--review", "true")
    assert result.returncode == 0, result.stderr
    consumer = root / "consumer"
    consumer.mkdir()
    (consumer / "worker.toml").write_text(
        "# Consumer policy\n" + (seed / "worker.toml").read_text()
    )
    (consumer / "src").mkdir()
    (consumer / "src/value.py").write_text("value = 1\n")
    return consumer


def test_setup_prepares_and_repeats_without_losing_settings(worker: Path, tmp_path: Path) -> None:
    root = declaration(worker, tmp_path)
    (root / ".codex").mkdir()
    settings = "# Keep this comment\nmodel = 'consumer-model'\n[features]\nhooks = true # enabled\n"
    (root / ".codex/config.toml").write_text(settings)
    (root / ".codex/config.toml").chmod(0o600)
    result = invoke(worker, root, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    actual = (root / ".codex/config.toml").read_text()
    assert actual.startswith(settings)
    assert (root / ".codex/config.toml").stat().st_mode & 0o777 == 0o600
    assert (root / "worker.toml").read_text().startswith("# Consumer policy\n")
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
    review = root / ".worker/review/config/review.toml"
    review.write_text(
        review.read_text().replace('model = "gpt-5.6-luna"', 'model = "consumer-model"')
    )
    state = root / "memory/State.md"
    state.write_text(state.read_text().replace("Not recorded.", "Consumer-owned state."))
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    assert file_contents(root) == before


def test_setup_rejects_conflicting_assets_without_writing(worker: Path, tmp_path: Path) -> None:
    root = declaration(worker, tmp_path)
    assert invoke(worker, root, "setup").returncode == 0
    skill = root / ".worker/skills/repair/SKILL.md"
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
    config = root / "worker.toml"
    config.write_text(
        config.read_text().replace("[capabilities]", "[capabilities]\nunknown = true")
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
    config = root / "worker.toml"
    config.write_text(
        config.read_text().replace('foreground = "process-group"', 'foreground = "systemd"')
    )
    launcher = tmp_path / "systemd-run"
    launcher.write_text("#!/bin/sh\necho 'scope unavailable' >&2\nexit 1\n")
    launcher.chmod(0o755)
    monkeypatch.setenv("PATH", str(tmp_path) + os.pathsep + os.environ["PATH"])
    result = invoke(worker, root, "setup")
    assert result.returncode != 0
    assert '"available":false' in result.stdout
    assert "process scope capability" in result.stdout


def delegated_project(worker: Path, root: Path) -> Path:
    consumer = declaration(worker, root)
    config = consumer / "worker.toml"
    config.write_text(
        config.read_text() + '\n[capabilities.delegation]\nconfig="agents/profiles.toml"\n'
    )
    agents = consumer / "agents"
    agents.mkdir()
    (agents / "prompt.md").write_text("Read the task and return its required JSON result.\n")
    (agents / "profiles.toml").write_text(
        'schema_version=1\n[profiles.reader]\nfrontend="codex"\nmodel="consumer-model"\n'
        'reasoning_effort="high"\nmode="read"\nprompt="prompt.md"\n'
        'visible_paths=["src/**"]\ntimeout_seconds=30\n'
        '[profiles.reader.credentials.env]\nOPENAI_API_KEY="CONSUMER_KEY"\n'
    )
    return consumer


def test_setup_registers_configured_delegation_and_preserves_resources(
    worker: Path, tmp_path: Path
) -> None:
    root = delegated_project(worker, tmp_path)
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
    installed = root / ".worker/bin/discipline-worker"
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
