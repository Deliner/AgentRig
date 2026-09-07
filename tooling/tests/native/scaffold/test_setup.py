import hashlib
import json
import os
import select
import subprocess
import time
import tomllib
from pathlib import Path

import pytest
import yaml
from support import CONFIG, file_contents, invoke, project, update_config


@pytest.mark.parametrize("backend", ["git", "mercurial"])
@pytest.mark.parametrize(
    "case",
    [
        ("main", True, True),
        ("main line", False, True),
        ("HEAD", False, True),
        ("tip", True, False),
        ("null", True, False),
        ("123", True, False),
        ("+123", True, False),
        ("12_3", True, True),
        ("bad:name", False, False),
        ("", False, False),
        (" main", False, False),
        ("main ", False, False),
        ("main\v", False, False),
        ("main\nline", False, False),
    ],
)
def test_configuration_uses_native_vcs_names(
    worker: Path, tmp_path: Path, backend: str, case: tuple[str, bool, bool]
) -> None:
    name, git_valid, hg_valid = case
    project(tmp_path, CONFIG)
    update_config(tmp_path / "agentrig.yaml", git={"backend": backend, "base": name})
    result = invoke(worker, tmp_path, "config-check")
    using_git = backend == "git"
    valid = git_valid if using_git else hg_valid
    assert result.returncode == (0 if valid else 2), result.stderr
    invalid = not valid
    if invalid:
        assert "vcs.base" in result.stderr
    assert not (tmp_path / ".git").exists() and not (tmp_path / ".hg").exists()


def test_mercurial_setup_retains_configured_spaces(worker: Path, tmp_path: Path) -> None:
    result = invoke(
        worker, tmp_path, "init", "--vcs", "mercurial", "--base", "main line", "--prefix", "task "
    )
    assert result.returncode == 0, result.stderr
    assert invoke(worker, tmp_path, "setup").returncode == 0
    assert subprocess.check_output(["hg", "branch"], cwd=tmp_path, text=True).strip() == "main line"
    assert invoke(worker, tmp_path, "setup").returncode == 0
    # Bootstrap generated files on a native feature branch before delivery commands.
    subprocess.run(["hg", "branch", "task product"], cwd=tmp_path, check=True, capture_output=True)
    subprocess.run(["hg", "add"], cwd=tmp_path, check=True, capture_output=True)
    committed = subprocess.run(
        ["hg", "commit", "-m", "bootstrap", "-u", "Test"],
        cwd=tmp_path,
        capture_output=True,
        text=True,
    )
    assert committed.returncode == 0, committed.stdout + committed.stderr
    update_config(tmp_path / "agentrig.yaml", vcs={"prefix": " leading/"})
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2 and "vcs.prefix" in result.stderr


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
    assert preview["registrations"]["vcs"]["hooks_path"] == "preview rig/hooks"
    assert preview["registrations"]["vcs"]["initialize"] is True
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


def interactive(worker: Path, root: Path, answers: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(worker), "init", "--interactive", "--root", str(root)],
        input="".join(answer + "\n" for answer in answers),
        capture_output=True,
        text=True,
        check=False,
    )


@pytest.mark.parametrize("language", ["python", "rust"])
@pytest.mark.parametrize("vcs", ["git", "mercurial"])
def test_interactive_init_matches_declarative_setup(
    worker: Path, tmp_path: Path, language: str, vcs: str
) -> None:
    target = tmp_path / "interactive"
    answers = [""] * 13
    answers[1] = language
    answers[9] = vcs
    answers[-1] = "yes"
    result = interactive(worker, target, answers)
    assert result.returncode == 0, result.stdout + result.stderr
    ordinary = tmp_path / "ordinary"
    ordinary.mkdir()
    assert invoke(worker, ordinary, "init", "--language", language, "--vcs", vcs).returncode == 0
    assert invoke(worker, ordinary, "setup").returncode == 0
    for name in [
        "agentrig.yaml",
        "AGENTS.md",
        "justfile",
        ".codex/config.toml",
        ".agentrig/manifest.json",
    ]:
        assert (target / name).read_bytes() == (ordinary / name).read_bytes(), name
    assert "Configuration for" in result.stdout
    assert '"preview": true' in result.stdout
    assert invoke(target / ".agentrig/bin/agentrig", target, "config-check").returncode == 0


@pytest.mark.parametrize("answers", [[], ["cancel"], [""] * 12, [""] * 12 + ["no"]])
def test_interactive_cancellation_leaves_no_target(
    worker: Path, tmp_path: Path, answers: list[str]
) -> None:
    target = tmp_path / "missing" / "consumer"
    result = interactive(worker, target, answers)
    assert result.returncode == 0, result.stdout + result.stderr
    assert "cancelled" in result.stdout
    assert not (tmp_path / "missing").exists()


def test_interactive_invalid_selection_preserves_existing_files(
    worker: Path, tmp_path: Path
) -> None:
    (tmp_path / "user.txt").write_text("Keep user content.\n")
    before = file_contents(tmp_path)
    answers = [""] * 13
    answers[1] = "unsupported-language"
    result = interactive(worker, tmp_path, answers)
    assert result.returncode == 2
    assert "python or rust" in result.stderr
    assert file_contents(tmp_path) == before


def test_interactive_selects_layout_checks_review_and_delegation(
    worker: Path, tmp_path: Path
) -> None:
    root = delegated_project(worker, tmp_path)
    (root / "agentrig.yaml").unlink()
    answers = [""] * 13
    answers[2] = "wizard rig"
    answers[4] = "notes"
    answers[8] = "true"
    answers[10] = "agents/profiles.yaml"
    answers[11] = "lint,memory"
    answers[12] = "yes"
    result = interactive(worker, root, answers)
    assert result.returncode == 0, result.stdout + result.stderr
    config = yaml.safe_load((root / "agentrig.yaml").read_text())
    assert config["paths"]["service"] == "wizard rig"
    assert config["paths"]["skills"] == "wizard rig/skills"
    assert config["paths"]["memory"] == "notes"
    assert {check["id"] for check in config["checks"]} == {"lint", "memory"}
    settings = tomllib.loads((root / ".codex/config.toml").read_text())
    assert set(settings["mcp_servers"]) == {"worker_review", "worker_delegation"}
    installed = root / "wizard rig/bin/agentrig"
    assert invoke(installed, root, "delegate", "config-check").returncode == 0
    assert invoke(installed, root, "review", "config-check").returncode == 0


def wait_for_preview(process: subprocess.Popen[bytes]) -> None:
    assert process.stdout is not None
    output = b""
    deadline = time.monotonic() + 10
    while b'"preview": true' not in output:
        ready, _, _ = select.select([process.stdout], [], [], max(0, deadline - time.monotonic()))
        assert ready, output.decode(errors="replace")
        chunk = os.read(process.stdout.fileno(), 65536)
        assert chunk, output.decode(errors="replace")
        output += chunk


def test_interactive_preserves_changes_after_preview(worker: Path, tmp_path: Path) -> None:
    with subprocess.Popen(
        [str(worker), "init", "--interactive", "--root", str(tmp_path)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    ) as process:
        try:
            assert process.stdin is not None
            process.stdin.write(b"\n" * 12)
            process.stdin.flush()
            wait_for_preview(process)
            note = tmp_path / "AGENTS.md"
            note.write_text("User change after preview.\n")
            output, errors = process.communicate(b"yes\n", timeout=10)
            assert process.returncode == 2, output + errors
            assert b"setup input changed" in errors
            assert note.read_text() == "User change after preview.\n"
            assert not (tmp_path / ".git").exists()
            assert not (tmp_path / "agentrig.yaml").exists()
        finally:
            alive = process.poll() is None
            if alive:
                process.kill()
            process.wait(timeout=10)


@pytest.mark.parametrize("wizard", [True, False])
def test_init_requires_explicit_legacy_migration(
    worker: Path, tmp_path: Path, wizard: bool
) -> None:
    legacy = tmp_path / "worker.toml"
    legacy.write_text("# Preserve the legacy declaration for explicit migration.\n")
    before = file_contents(tmp_path)
    if wizard:
        result = interactive(worker, tmp_path, [""] * 12 + ["yes"])
    else:
        result = invoke(worker, tmp_path, "init")
    assert result.returncode == 2
    assert "explicit upgrade" in result.stderr
    assert file_contents(tmp_path) == before
