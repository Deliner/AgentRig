import json
import shutil
import subprocess
import tomllib
from pathlib import Path
from typing import Any

import pytest
import yaml

from .test_upgrade_apply import snapshot
from .test_upgrade_plan import invoke


def consumer(
    worker: Path, root: Path, review: str = "false", backend: str = "git"
) -> tuple[Path, Path]:
    declaration = source_configuration(worker, root, review, backend)
    target = install_consumer(worker, root, declaration)
    return target, declaration


def source_configuration(worker: Path, root: Path, review: str, backend: str) -> Path:
    source = root / "source"
    source.mkdir()
    private = backend == "private"
    native = "mercurial" if private else backend
    result = invoke(
        worker, source, "init", "--service", "custom rig", "--review", review, "--vcs", native
    )
    assert result.returncode == 0, result.stderr
    declaration = source / "agentrig.yaml"
    config = yaml.safe_load(declaration.read_text())
    if private:
        adapter = Path(__file__).resolve().parents[4] / "examples/external_vcs.py"
        config["vcs"]["backend"] = {"command": ["python3", "-B", str(adapter)]}
    config["commands"]["hello"] = {"argv": ["python3", "-c", "print('before')"]}
    declaration.write_text(yaml.safe_dump(config))
    return declaration


def install_consumer(worker: Path, root: Path, declaration: Path) -> Path:
    target = root / "target"
    target.mkdir()
    result = invoke(worker, target, "setup", "--config", str(declaration))
    assert result.returncode == 0, result.stdout + result.stderr
    return target


def client_consumer(worker: Path, root: Path, frontend: str) -> tuple[Path, Path]:
    declaration = source_configuration(worker, root, "false", "git")
    config = yaml.safe_load(declaration.read_text())
    config["frontend"] = frontend
    config["agent"] = {
        "model": "before-model",
        "reasoning_effort": "low",
        "api": {"key_env": "OLD_KEY", "base_url": "https://before.example"},
    }
    config["environment"] = {
        "programs": {"probe": "probe.sh"},
        "hooks": {
            "probe": {
                "event": "SessionStart",
                "program": "probe",
                "args": ["before"],
                "timeout_seconds": 10,
            }
        },
        "mcp_servers": {"probe": {"program": "probe", "args": ["before"]}},
    }
    program = declaration.parent / "probe.sh"
    program.write_text("#!/bin/sh\nprintf '%s' \"$1\"\n")
    program.chmod(0o755)
    declaration.write_text(yaml.safe_dump(config))
    return install_consumer(worker, root, declaration), declaration


def client_preferences(target: Path, frontend: str) -> None:
    codex = frontend == "codex"
    hook_file = target / ".claude/settings.json"
    if codex:
        settings = target / ".codex/config.toml"
        settings.write_text(
            "# Keep preferences\nproject_doc_max_bytes = 4096\n"
            + settings.read_text()
            + '\n[mcp_servers.foreign]\ncommand = "external"\n'
        )
        hook_file = target / ".codex/hooks.json"
    else:
        settings = hook_file
        mcp = target / ".mcp.json"
        data = json.loads(mcp.read_text())
        data["mcpServers"]["foreign"] = {"command": "external"}
        mcp.write_text(json.dumps(data))
    data = json.loads(hook_file.read_text())
    data["hooks"]["SessionStart"].append(
        {"matcher": "foreign", "hooks": [{"type": "command", "command": "true"}]}
    )
    claude = not codex
    if claude:
        data.setdefault("env", {})["PRESERVE_ME"] = "kept"
    hook_file.write_text(json.dumps(data))
    settings.chmod(0o600)


@pytest.mark.parametrize("frontend", ["codex", "claude-code"])
@pytest.mark.parametrize("remove", [False, True])
def test_client_configuration_update_preserves_preferences_and_rolls_back(
    worker: Path, tmp_path: Path, frontend: str, remove: bool
) -> None:
    target, declaration = client_consumer(worker, tmp_path, frontend)
    client_preferences(target, frontend)
    config = yaml.safe_load(declaration.read_text())
    if remove:
        config.pop("agent")
    else:
        config["agent"] = {
            "model": "after-model",
            "reasoning_effort": "high",
            "api": {"key_env": "NEW_KEY"},
        }
    config["environment"]["mcp_servers"] = {"renamed": {"program": "probe", "args": ["after"]}}
    config["environment"]["hooks"]["probe"]["args"] = ["after"]
    config["environment"]["hooks"]["probe"]["timeout_seconds"] = 20
    declaration.write_text(yaml.safe_dump(config))
    path = update(worker, target, declaration)
    plan = json.loads(path.read_text())
    original = snapshot(target, list(plan["files"]))
    assert any(change["action"] == "conflict" for change in plan["files"].values())
    assert invoke(worker, target, "upgrade", "apply", str(path)).returncode == 2
    assert snapshot(target, list(plan["files"])) == original
    resolve_update_conflicts(path)
    shutil.rmtree(declaration.parent)
    result = invoke(worker, target, "upgrade", "apply", str(path))
    assert result.returncode == 0, result.stdout + result.stderr
    verify_client_update(target, frontend, remove)
    before_repeat = snapshot(target, list(plan["files"]))
    assert invoke(worker, target, "setup").returncode == 0
    after_repeat = snapshot(target, list(plan["files"]))
    for relative, before in before_repeat.items():
        assert after_repeat[relative] == before, relative
    rolled_back = invoke(worker, target, "upgrade", "rollback")
    assert rolled_back.returncode == 0, rolled_back.stdout + rolled_back.stderr
    assert snapshot(target, list(plan["files"])) == original


def verify_client_update(target: Path, frontend: str, removed: bool) -> None:
    codex = frontend == "codex"
    if codex:
        settings_path = target / ".codex/config.toml"
        settings = tomllib.loads(settings_path.read_text())
        assert settings["project_doc_max_bytes"] == 4096
        assert "# Keep preferences" in settings_path.read_text()
        servers = settings["mcp_servers"]
        assert servers["probe"]["enabled"] is False
        provider = settings.get("model_providers", {}).get("agentrig_api", {})
        assert provider.get("env_key") == (None if removed else "NEW_KEY")
        assert provider.get("base_url") == (None if removed else "https://api.openai.com/v1")
        hooks = json.loads((target / ".codex/hooks.json").read_text())
    else:
        settings_path = target / ".claude/settings.json"
        settings = json.loads(settings_path.read_text())
        servers = json.loads((target / ".mcp.json").read_text())["mcpServers"]
        assert "probe" not in servers
        assert "ANTHROPIC_BASE_URL" not in settings["env"]
        assert settings["env"]["PRESERVE_ME"] == "kept"
        assert ("NEW_KEY" in settings.get("apiKeyHelper", "")) != removed
        hooks = settings
    assert settings.get("model") == (None if removed else "after-model")
    effort = "model_reasoning_effort" if codex else "effortLevel"
    assert settings.get(effort) == (None if removed else "high")
    assert settings_path.stat().st_mode & 0o777 == 0o600
    assert servers["foreign"] == {"command": "external"}
    assert servers["renamed"]["command"] == "sh"
    groups = hooks["hooks"]["SessionStart"]
    assert any(group.get("matcher") == "foreign" for group in groups)
    verify_updated_hook(target, groups)


def resolve_update_conflicts(path: Path) -> None:
    plan = json.loads(path.read_text())
    for change in plan["files"].values():
        conflict = change["action"] == "conflict"
        if conflict:
            change["resolution"] = "replace"
    path.write_text(json.dumps(plan))


def verify_updated_hook(target: Path, groups: list[dict[str, Any]]) -> None:
    routes = [route for group in groups for route in group["hooks"]]
    managed = list(filter(lambda route: "environment-hook" in route["command"], routes))
    assert len(managed) == 1
    assert managed[0]["timeout"] == 20
    result = subprocess.run(
        ["sh", "-c", managed[0]["command"]],
        cwd=target,
        input='{"hook_event_name":"SessionStart"}',
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    assert result.stdout == "after"


@pytest.mark.parametrize("frontend", ["codex", "claude-code"])
def test_update_rejects_disabled_client_hooks_before_writing(
    worker: Path, tmp_path: Path, frontend: str
) -> None:
    target, declaration = client_consumer(worker, tmp_path, frontend)
    codex = frontend == "codex"
    if codex:
        path = target / ".codex/config.toml"
        path.write_text(path.read_text().replace("hooks = true", "hooks = false"))
    else:
        path = target / ".claude/settings.json"
        settings = json.loads(path.read_text())
        settings["disableAllHooks"] = True
        path.write_text(json.dumps(settings))
    receipt = "custom rig/manifest.json"
    paths = [*json.loads((target / receipt).read_text())["files"], receipt]
    original = snapshot(target, paths)
    result = invoke(worker, target, "upgrade", "plan", "--config", str(declaration))
    assert result.returncode == 2, result.stdout + result.stderr
    assert "hooks" in result.stderr.lower()
    assert snapshot(target, paths) == original


@pytest.mark.parametrize("frontend", ["codex", "claude-code"])
def test_frontend_update_preserves_existing_native_preferences(
    worker: Path, tmp_path: Path, frontend: str
) -> None:
    target, declaration = client_consumer(worker, tmp_path, frontend)
    previous_codex = frontend == "codex"
    selected = "claude-code" if previous_codex else "codex"
    codex = selected == "codex"
    relative = ".codex/config.toml" if codex else ".claude/settings.json"
    preferences = target / relative
    preferences.parent.mkdir(exist_ok=True)
    contents = 'model = "native-choice"\n' if codex else '{"model":"native-choice"}'
    preferences.write_text(contents)
    preferences.chmod(0o600)
    config = yaml.safe_load(declaration.read_text())
    config["frontend"] = selected
    config.pop("agent")
    declaration.write_text(yaml.safe_dump(config))
    path = update(worker, target, declaration)
    plan = json.loads(path.read_text())
    original = snapshot(target, list(plan["files"]))
    resolve_update_conflicts(path)
    result = invoke(worker, target, "upgrade", "apply", str(path))
    assert result.returncode == 0, result.stdout + result.stderr
    parse = tomllib.loads if codex else json.loads
    assert parse(preferences.read_text())["model"] == "native-choice"
    assert preferences.stat().st_mode & 0o777 == 0o600
    assert yaml.safe_load((target / "agentrig.yaml").read_text())["frontend"] == selected
    assert invoke(worker, target, "setup").returncode == 0
    rolled_back = invoke(worker, target, "upgrade", "rollback")
    assert rolled_back.returncode == 0, rolled_back.stdout + rolled_back.stderr
    assert snapshot(target, list(plan["files"])) == original


def update(worker: Path, target: Path, source: Path) -> Path:
    result = invoke(worker, target, "upgrade", "plan", "--config", str(source))
    assert result.returncode == 0, result.stdout + result.stderr
    return Path(result.stdout.rsplit("Plan: ", 1)[1].strip())


@pytest.mark.parametrize("backend", ["git", "mercurial", "private"])
def test_configuration_update_and_rollback(worker: Path, tmp_path: Path, backend: str) -> None:
    target, declaration = consumer(worker, tmp_path, backend=backend)
    metadata = {"git": ".git/config", "mercurial": ".hg/hgrc", "private": ".hg/hgrc"}[backend]
    registration = (target / metadata).read_bytes()
    declaration.write_text(declaration.read_text().replace("print('before')", "print('after')"))
    skill = declaration.parent / "custom rig/skills/repair/SKILL.md"
    skill.write_text(skill.read_text() + "\nNew upstream instruction.\n")
    state = target / "memory/State.md"
    state.write_text(state.read_text().replace("Not recorded.", "Consumer recovery context."))
    path = update(worker, target, declaration)
    plan = json.loads(path.read_text())
    assert plan["service"] == "custom rig"
    assert plan["from_version"] == plan["to_version"] == "0.3.0"
    assert plan["files"]["memory/State.md"]["action"] == "keep"
    assert all(change["action"] != "conflict" for change in plan["files"].values())
    original = snapshot(target, list(plan["files"]))
    lines = (path.parent / "diff.txt").read_text().splitlines()
    assert any(line.startswith("+") and "print('after')" in line for line in lines)
    result = invoke(worker, target, "upgrade", "apply", str(path))
    assert result.returncode == 0, result.stdout + result.stderr
    result = invoke(worker, target, "run", "hello")
    assert result.returncode == 0, result.stderr
    assert "after" in result.stdout
    assert "New upstream instruction." in (target / "custom rig/skills/repair/SKILL.md").read_text()
    assert state.read_bytes() == original["memory/State.md"]
    expected = yaml.safe_load(declaration.read_text())["vcs"]["backend"]
    assert yaml.safe_load((target / "agentrig.yaml").read_text())["vcs"]["backend"] == expected
    assert (target / metadata).read_bytes() == registration
    result = invoke(worker, target, "upgrade", "rollback")
    assert result.returncode == 0, result.stdout + result.stderr
    assert snapshot(target, list(plan["files"])) == original
    assert "before" in invoke(worker, target, "run", "hello").stdout
    assert (target / metadata).read_bytes() == registration


@pytest.mark.parametrize("backend", ["git", "private"])
def test_configuration_update_reports_local_conflicts(
    worker: Path, tmp_path: Path, backend: str
) -> None:
    target, declaration = consumer(worker, tmp_path, backend=backend)
    skill_path = "custom rig/skills/repair/SKILL.md"
    local = target / skill_path
    local.write_text(local.read_text() + "\nLocal instruction.\n")
    source = declaration.parent / skill_path
    source.write_text(source.read_text() + "\nUpdated upstream instruction.\n")
    path = update(worker, target, declaration)
    plan = json.loads(path.read_text())
    assert plan["files"][skill_path]["action"] == "conflict"
    original = local.read_bytes()
    result = invoke(worker, target, "upgrade", "apply", str(path))
    assert result.returncode == 2
    assert "unresolved conflict" in result.stderr
    assert local.read_bytes() == original
    plan["files"][skill_path]["resolution"] = "keep"
    path.write_text(json.dumps(plan))
    result = invoke(worker, target, "upgrade", "apply", str(path))
    assert result.returncode == 0, result.stdout + result.stderr
    assert local.read_bytes() == original


def test_successive_configuration_updates_keep_recovery(worker: Path, tmp_path: Path) -> None:
    target, declaration = consumer(worker, tmp_path)
    declaration.write_text(declaration.read_text().replace("print('before')", "print('second')"))
    first = update(worker, target, declaration)
    result = invoke(worker, target, "upgrade", "apply", str(first))
    assert result.returncode == 0, result.stdout + result.stderr
    declaration.write_text(declaration.read_text().replace("print('second')", "print('third')"))
    second = update(worker, target, declaration)
    shutil.rmtree(declaration.parent)
    result = invoke(worker, target, "upgrade", "apply", str(second))
    assert result.returncode == 0, result.stdout + result.stderr
    assert "third" in invoke(worker, target, "run", "hello").stdout
    assert invoke(worker, target, "upgrade", "rollback").returncode == 0
    assert "second" in invoke(worker, target, "run", "hello").stdout


def test_stale_update_preserves_previous_recovery(worker: Path, tmp_path: Path) -> None:
    target, declaration = consumer(worker, tmp_path)
    declaration.write_text(declaration.read_text().replace("print('before')", "print('second')"))
    first = update(worker, target, declaration)
    assert invoke(worker, target, "upgrade", "apply", str(first)).returncode == 0
    declaration.write_text(declaration.read_text().replace("print('second')", "print('third')"))
    second = update(worker, target, declaration)
    state = target / "memory/State.md"
    state.write_text(state.read_text() + "\nLocal recovery note.\n")
    result = invoke(worker, target, "upgrade", "apply", str(second))
    assert result.returncode == 2
    assert "file changed since planning" in result.stderr
    journal = json.loads((second.parent.parent / "operation/journal.json").read_text())
    assert journal["plan"] == str(first)
    assert journal["phase"] == "applied"


def test_update_preserves_codex_settings_and_changes_managed_registration(
    worker: Path, tmp_path: Path
) -> None:
    target, declaration = consumer(worker, tmp_path, "true")
    settings = target / ".codex/config.toml"
    settings.write_text('model = "consumer-model"\n# Keep preferences\n' + settings.read_text())
    settings.chmod(0o600)
    original = settings.read_bytes()
    review = declaration.parent / "custom rig/review/config/review.yaml"
    config = yaml.safe_load(review.read_text())
    config["runner"]["timeout_seconds"] = 123
    review.write_text(yaml.safe_dump(config))
    path = update(worker, target, declaration)
    plan = json.loads(path.read_text())
    change = plan["files"][".codex/config.toml"]
    assert change["action"] == "conflict"
    change["resolution"] = "replace"
    path.write_text(json.dumps(plan))
    result = invoke(worker, target, "upgrade", "apply", str(path))
    assert result.returncode == 0, result.stdout + result.stderr
    actual = tomllib.loads(settings.read_text())
    assert actual["model"] == "consumer-model"
    assert "# Keep preferences" in settings.read_text()
    assert actual["mcp_servers"]["worker_review"]["tool_timeout_sec"] == 183
    assert settings.stat().st_mode & 0o777 == 0o600
    assert invoke(worker, target, "upgrade", "rollback").returncode == 0
    assert settings.read_bytes() == original
