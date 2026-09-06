# DECISION: D015
# DECISION: D014
# DECISION: D013
# DECISION: D005
import json
from pathlib import Path
from typing import Any

import pytest
from support import invoke


@pytest.mark.parametrize(
    "edit",
    [
        ("Edit", {"file_path": "Ledger/Plan.md"}, ["edit-plan"]),
        ("Write", {"path": "Ledger/State.md"}, ["edit-state"]),
        ("Edit", {"file_path": "Ledger/Decisions.md"}, ["edit-decisions"]),
        ("Write", {"file_path": "Ledger/Invariants.md"}, ["edit-invariants"]),
        ("Edit", {"file_path": "Ledger/Plan/001.md"}, ["edit-plan"]),
        ("Edit", {"file_path": "Ledger/Decisions/001.md"}, ["edit-decisions"]),
        ("Edit", {"file_path": "Ledger/Invariants/001.md"}, ["edit-invariants"]),
        ("Edit", {"file_path": "Project/State.md"}, []),
        ("Edit", {"file_path": "Ledger/Plan-other.md"}, []),
        ("Edit", {"file_path": "Ledger/Plan/../State.md"}, ["edit-state"]),
        ("Edit", {"file_path": "{root}/Ledger/State.md"}, ["edit-state"]),
        ("Edit", {"file_path": "{outside}/Ledger/State.md"}, []),
        (
            "apply_patch",
            "*** Begin Patch\n*** Update File: Ledger/Plan.md\n"
            "*** Update File: Ledger/Plan/001.md\n*** Delete File: Ledger/State.md\n"
            "*** Add File: Ledger/Invariants/013.md\n*** End Patch",
            ["edit-plan", "edit-invariants", "edit-state"],
        ),
        (
            "apply_patch",
            {
                "patch": "*** Update File: Ledger/Decisions/001.md\n"
                "*** Move to: Ledger/Invariants/001.md\n"
            },
            ["edit-decisions", "edit-invariants"],
        ),
        (
            "apply_patch",
            {"input": "*** Add File: Ledger/State.md\n+# State\n"},
            ["edit-state"],
        ),
        (
            "apply_patch",
            {
                "patch_text": "*** Update File: Project/example.md\n"
                "+*** Update File: Ledger/Plan.md\n"
            },
            [],
        ),
    ],
)
# INVARIANT: I012
def test_ledger_edit_guidance(
    worker: Path, tmp_path: Path, edit: tuple[str, Any, list[str]]
) -> None:
    tool, payload, expected = edit
    assert (
        invoke(worker, tmp_path, "init", "--memory", "Ledger", "--skills", "guides").returncode == 0
    )
    payload = json.loads(
        json.dumps(payload)
        .replace("{root}", str(tmp_path))
        .replace("{outside}", str(tmp_path.parent))
    )
    event = {
        "hook_event_name": "PreToolUse",
        "tool_name": tool,
        "tool_input": payload,
        "cwd": str(tmp_path),
    }
    output = invoke(worker, tmp_path, "hook", input=json.dumps(event))
    result = json.loads(output.stdout) if output.stdout else None
    no_guidance_expected = not expected
    if no_guidance_expected:
        assert result is None
        return
    assert result is not None
    output = result["hookSpecificOutput"]
    assert "permissionDecision" not in output
    message = output["additionalContext"]
    for skill in ("edit-plan", "edit-decisions", "edit-invariants", "edit-state"):
        path = tmp_path / "guides" / skill / "SKILL.md"
        assert path.is_file()
        assert message.count(str(path)) == int(skill in expected)


@pytest.mark.parametrize(
    "command",
    [
        "git status",
        "just unknown",
        "just list && git status",
        'just run write echo "$(pwd)"',
        "just --justfile other list",
        "just list\npwd",
        "just list write echo bypass",
    ],
)
# INVARIANT: I005
def test_shell_guard_rejects_bypass(worker: Path, tmp_path: Path, command: str) -> None:
    assert invoke(worker, tmp_path, "init").returncode == 0
    event = {
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": command},
    }
    result = invoke(worker, tmp_path, "hook", input=json.dumps(event))
    assert result.returncode == 0
    assert json.loads(result.stdout)["hookSpecificOutput"]["permissionDecision"] == "deny"


def test_nested_cwd_and_symlinks(worker: Path, tmp_path: Path) -> None:
    assert invoke(worker, tmp_path, "init", "--memory", "notes").returncode == 0
    event = {
        "hook_event_name": "PreToolUse",
        "tool_name": "Edit",
        "cwd": str(tmp_path / "notes"),
        "tool_input": {"file_path": "Plan.md"},
    }
    assert "edit-plan" in invoke(worker, tmp_path, "hook", input=json.dumps(event)).stdout
    path = tmp_path / "notes/Plan.md"
    path.unlink()
    path.symlink_to(tmp_path.parent / "outside.md")
    assert invoke(worker, tmp_path, "hook", input=json.dumps(event)).stdout == ""


def test_storage_failure_keeps_recovery_context(worker: Path, tmp_path: Path) -> None:
    assert invoke(worker, tmp_path, "init").returncode == 0
    (tmp_path / ".worker/runtime").write_text("occupied")
    result = invoke(worker, tmp_path, "hook", input=json.dumps({"hook_event_name": "SessionStart"}))
    assert "memory/State.md" in result.stdout
    assert "FULL_REFRESH_REQUIRED" in result.stdout
    assert "session reminder state" in result.stderr


def test_hook_requires_project_configuration(worker: Path, tmp_path: Path) -> None:
    result = invoke(worker, tmp_path, "hook", input=json.dumps({"hook_event_name": "SessionStart"}))
    assert "agentrig.yaml" in result.stdout
    assert "deny" in result.stdout


def test_shell_catalog_and_recipe_listing(worker: Path, tmp_path: Path) -> None:
    assert invoke(worker, tmp_path, "init").returncode == 0
    for command in [
        "just --list",
        "just list",
        "just lint-rule function-lines --example",
        "just lint-explain src/example.py --json",
        "just run read -- git status",
        "just run write -- echo ok",
    ]:
        event = {
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": command},
        }
        result = invoke(worker, tmp_path, "hook", input=json.dumps(event))
        assert "deny" not in result.stdout
