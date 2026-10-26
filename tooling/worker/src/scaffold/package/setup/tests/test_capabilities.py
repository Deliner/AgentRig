import json
import subprocess
from pathlib import Path

import pytest

from tooling.worker.src.scaffold.testing.consumer import (
    CONFIG,
    file_contents,
    invoke,
    project,
)


def test_disabled_lint_requires_consistent_gate(worker: Path, tmp_path: Path) -> None:
    source = CONFIG + "\ncapabilities:\n  lint: false\n"
    project(tmp_path, source)
    (tmp_path / "lint.yaml").unlink()
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 0, result.stderr
    result = invoke(worker, tmp_path, "lint")
    assert result.returncode == 2
    assert "capabilities.lint is disabled" in result.stderr
    (tmp_path / "agentrig.yaml").write_text(
        source + '\nchecks:\n- id: "lint"\n  kind: "lint"\n  skill: "guides/repair/SKILL.md"\n'
    )
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "lint check is configured" in result.stderr


@pytest.mark.parametrize("capability", ["unknown", "review", "delegation"])
def test_unknown_capability_is_actionable(worker: Path, tmp_path: Path, capability: str) -> None:
    project(tmp_path, CONFIG + f"\ncapabilities:\n  {capability}: true\n")
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert f"unknown field `{capability}`" in result.stderr


@pytest.mark.parametrize(
    "replacement", ['version: "1"', "version: 1\nversion: 1", "version: &schema 1"]
)
def test_root_yaml_is_strict(worker: Path, tmp_path: Path, replacement: str) -> None:
    project(tmp_path, CONFIG.replace("version: 1", replacement, 1))
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "agentrig.yaml" in result.stderr and "ACTION:" in result.stderr


def test_claude_init_registers_guarded_hooks(worker: Path, tmp_path: Path) -> None:
    result = invoke(worker, tmp_path, "init", "--frontend", "claude-code")
    assert result.returncode == 0, result.stderr
    assert not (tmp_path / ".codex").exists()
    assert not (tmp_path / ".mcp.json").exists()
    assert (tmp_path / "CLAUDE.md").read_text() == "@AGENTS.md\n"
    settings = json.loads((tmp_path / ".claude/settings.json").read_text())
    command = settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"]
    event = dict(
        hook_event_name="PreToolUse",
        tool_name="Bash",
        tool_input={"command": "echo bypass"},
        cwd=str(tmp_path),
    )
    hook = subprocess.run(
        ["sh", "-c", command],
        cwd=tmp_path,
        input=json.dumps(event),
        text=True,
        capture_output=True,
        timeout=20,
    )
    assert hook.returncode == 0, hook.stderr
    assert json.loads(hook.stdout)["hookSpecificOutput"]["permissionDecision"] == "deny"


@pytest.mark.parametrize("existing", [".claude/settings.json", "CLAUDE.md"])
def test_claude_init_collision_preserves_files(worker: Path, tmp_path: Path, existing: str) -> None:
    path = tmp_path / existing
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text('{"model": "consumer-model"}\n')
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "init", "--frontend", "claude-code")
    assert result.returncode == 2
    assert "collision" in result.stderr
    assert file_contents(tmp_path) == before
