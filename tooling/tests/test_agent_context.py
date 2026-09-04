from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any

import pytest
from agent_context import LEDGER_SKILLS, dispatch

# DECISION: D013
# DECISION: D014

ROOT = Path(__file__).parents[2]


@pytest.mark.parametrize(
    ("tool", "payload", "expected"),
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
        ("Edit", {"file_path": str(ROOT / "Ledger/State.md")}, ["edit-state"]),
        ("Edit", {"file_path": str(ROOT.parent / "Ledger/State.md")}, []),
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
def test_ledger_edit_guidance(tool: str, payload: Any, expected: list[str]) -> None:
    event = {
        "hook_event_name": "PreToolUse",
        "tool_name": tool,
        "tool_input": payload,
        "cwd": str(ROOT),
    }
    result = dispatch(event)
    if not expected:
        assert result is None
        return
    assert result is not None
    output = result["hookSpecificOutput"]
    assert "permissionDecision" not in output
    message = output["additionalContext"]
    for skill in LEDGER_SKILLS.values():
        path = ROOT / ".agents/skills" / skill / "SKILL.md"
        assert path.is_file()
        assert message.count(str(path)) == int(skill in expected)


def test_nested_cwd_and_symlinks(tmp_path: Path) -> None:
    ledger = tmp_path / "Ledger"
    ledger.mkdir()
    event = {
        "hook_event_name": "PreToolUse",
        "tool_name": "Edit",
        "tool_input": {"file_path": "Plan.md"},
        "cwd": str(ledger),
    }
    assert "edit-plan" in json.dumps(dispatch(event, tmp_path))
    (ledger / "Plan.md").symlink_to(tmp_path.parent / "outside.md")
    assert dispatch(event, tmp_path) is None


@pytest.mark.parametrize("source", ["startup", "resume", "clear", "compact"])
def test_session_start_points_to_recovery(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, source: str
) -> None:
    monkeypatch.setenv("COMPLEXITY_DISCIPLINE_STATE_DIR", str(tmp_path / "reminders"))
    result = dispatch({"hook_event_name": "SessionStart", "source": source, "cwd": str(ROOT)})
    assert result is not None
    output = result["hookSpecificOutput"]
    assert output["hookEventName"] == "SessionStart"
    assert str(ROOT / "Ledger/State.md") in output["additionalContext"]
    assert str(ROOT / "Ledger/Plan.md") in output["additionalContext"]
    assert "FULL_REFRESH_REQUIRED" in output["additionalContext"]


def test_session_recovery_survives_reminder_storage_failure(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    storage = tmp_path / "not-a-directory"
    storage.write_text("occupied", encoding="utf-8")
    monkeypatch.setenv("COMPLEXITY_DISCIPLINE_STATE_DIR", str(storage))
    result = dispatch({"hook_event_name": "SessionStart"})
    assert result is not None
    assert "Ledger/State.md" in result["hookSpecificOutput"]["additionalContext"]


@pytest.mark.parametrize("tokens", [35_000, 140_000])
def test_complexity_denial_and_retry_preserve_ledger_guidance(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, tokens: int
) -> None:
    monkeypatch.setenv("COMPLEXITY_DISCIPLINE_STATE_DIR", str(tmp_path / "reminders"))
    transcript = tmp_path / "session.jsonl"
    transcript.write_text("", encoding="utf-8")
    event = {"cwd": str(ROOT), "transcript_path": str(transcript), "session_id": "test"}
    dispatch({**event, "hook_event_name": "SessionStart"})
    record = {
        "type": "event_msg",
        "payload": {"type": "token_count", "info": {"last_token_usage": {"input_tokens": tokens}}},
    }
    transcript.write_text(json.dumps(record) + "\n", encoding="utf-8")
    edit = {
        **event,
        "hook_event_name": "PreToolUse",
        "tool_name": "Write",
        "tool_input": {"file_path": "Ledger/State.md"},
    }
    result = dispatch(edit)
    assert result is not None
    output = result["hookSpecificOutput"]
    assert output["permissionDecision"] == "deny"
    assert "edit-state" in output["permissionDecisionReason"]
    assert ("FULL_REFRESH_REQUIRED" in output["permissionDecisionReason"]) == (tokens == 140_000)
    retry = dispatch(edit)
    assert retry is not None
    assert "permissionDecision" not in retry["hookSpecificOutput"]
    assert "edit-state" in retry["hookSpecificOutput"]["additionalContext"]


@pytest.mark.parametrize("tool", ["Bash", "Shell", "exec_command"])
def test_shell_guard_and_conditional_write_guidance(tool: str) -> None:
    event = {"hook_event_name": "PreToolUse", "tool_name": tool}
    for command in ("git status", "just unknown", "just status && git status"):
        result = dispatch({**event, "tool_input": {"cmd": command}})
        assert result is not None
        assert result["hookSpecificOutput"]["permissionDecision"] == "deny"
    assert dispatch({**event, "tool_input": {"command": "just status"}}) is None
    result = dispatch({**event, "tool_input": {"cmd": "just write -- python3 change.py"}})
    assert result is not None
    output = result["hookSpecificOutput"]
    assert "permissionDecision" not in output
    for skill in LEDGER_SKILLS.values():
        assert skill in output["additionalContext"]


def test_single_registered_entrypoint_and_json_protocol(tmp_path: Path) -> None:
    hooks = json.loads((ROOT / ".codex/hooks.json").read_text(encoding="utf-8"))["hooks"]
    assert len(hooks["PreToolUse"]) == 1
    for event in ("SessionStart", "PreToolUse"):
        assert len(hooks[event][0]["hooks"]) == 1
        assert "agent_context.py" in hooks[event][0]["hooks"][0]["command"]
    env = {**os.environ, "COMPLEXITY_DISCIPLINE_STATE_DIR": str(tmp_path / "state")}
    result = subprocess.run(
        [sys.executable, str(ROOT / ".codex/hooks/agent_context.py")],
        input=json.dumps(
            {
                "hook_event_name": "PreToolUse",
                "tool_name": "Edit",
                "tool_input": {"file_path": str(ROOT / "Ledger/Plan.md")},
            }
        ),
        text=True,
        capture_output=True,
        check=True,
        env=env,
    )
    assert result.stderr == ""
    output = json.loads(result.stdout)["hookSpecificOutput"]
    assert "edit-plan" in output["additionalContext"]
    assert "permissionDecision" not in output
