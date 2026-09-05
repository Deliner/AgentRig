from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Any

import pytest

# DECISION: D015
ROOT = Path(__file__).parents[3]


def native(worker: Path, event: dict[str, Any], state: Path) -> dict[str, Any] | None:
    result = subprocess.run(
        [str(worker), "hook", "--root", str(ROOT)],
        input=json.dumps(event),
        text=True,
        capture_output=True,
        check=True,
        env={**os.environ, "COMPLEXITY_DISCIPLINE_STATE_DIR": str(state)},
    )
    return json.loads(result.stdout) if result.stdout else None


@pytest.mark.parametrize(
    ("tool", "payload"),
    [
        ("Write", {"file_path": "Ledger/State.md"}),
        ("Edit", {"path": str(ROOT / "Ledger/Plan/001.md")}),
        ("Edit", {"path": "Project/State.md"}),
        ("apply_patch", "*** Update File: Ledger/Plan.md\n*** Move to: Ledger/Invariants/001.md\n"),
        ("apply_patch", {"patch": "*** Add File: Ledger/Decisions/015.md\n"}),
        ("exec_command", {"cmd": "just status"}),
        ("Shell", {"command": "just write -- python3 example.py"}),
    ],
)
# INVARIANT: I013
def test_native_hook_contract(worker: Path, tmp_path: Path, tool: str, payload: Any) -> None:
    event = {
        "hook_event_name": "PreToolUse",
        "tool_name": tool,
        "tool_input": payload,
        "cwd": str(ROOT),
    }
    output = native(worker, event, tmp_path)
    if tool == "exec_command" or payload == {"path": "Project/State.md"}:
        assert output is None
    else:
        assert output is not None
        context = output["hookSpecificOutput"]
        assert "permissionDecision" not in context
        assert "edit-" in context["additionalContext"]


@pytest.mark.parametrize(
    "command",
    [
        "git status",
        "just unknown",
        "just status && git status",
        'just write -- echo "$(pwd)"',
        "just --justfile alternate status",
        "just status\npwd",
    ],
)
def test_native_shell_guard(worker: Path, tmp_path: Path, command: str) -> None:
    output = native(
        worker,
        {"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": command}},
        tmp_path,
    )
    assert output is not None
    assert output["hookSpecificOutput"]["permissionDecision"] == "deny"


def test_reminder_transcript_and_state(worker: Path, tmp_path: Path) -> None:
    transcript = tmp_path / "transcript.jsonl"
    transcript.touch()
    state = tmp_path / "state"
    common = {"cwd": str(ROOT), "session_id": "reminders", "transcript_path": str(transcript)}
    start = native(worker, {**common, "hook_event_name": "SessionStart"}, state)
    assert start is not None
    assert "FULL_REFRESH_REQUIRED" in json.dumps(start)
    edit = {
        **common,
        "hook_event_name": "PreToolUse",
        "tool_name": "Write",
        "tool_input": {"file_path": "Ledger/State.md"},
    }
    for count, expected in [
        (34_999, None),
        (35_000, "CHECKPOINT"),
        (35_001, None),
        (70_000, "CHECKPOINT"),
        (70_001, None),
        (140_000, "FULL_REFRESH_REQUIRED"),
        (140_001, None),
        (100, None),
        (35_100, "CHECKPOINT"),
        (35_101, None),
    ]:
        with transcript.open("a") as stream:
            stream.write(
                json.dumps(
                    {
                        "type": "event_msg",
                        "payload": {
                            "type": "token_count",
                            "info": {"last_token_usage": {"input_tokens": count}},
                        },
                    }
                )
                + "\n"
            )
        result = native(worker, edit, state)
        assert result is not None
        output = result["hookSpecificOutput"]
        if expected is None:
            assert "permissionDecision" not in output
        else:
            assert output["permissionDecision"] == "deny"
            assert expected in output["permissionDecisionReason"]
        saved = json.loads(next(state.glob("*.json")).read_text())
        assert saved["latest_tokens"] == count
        assert saved["scan_offset"] == transcript.stat().st_size
    offset = transcript.stat().st_size
    with transcript.open("a") as stream:
        stream.write('{"type":')
    native(worker, edit, state)
    assert json.loads(next(state.glob("*.json")).read_text())["scan_offset"] == offset
    transcript.write_text("")
    native(worker, edit, state)
    assert json.loads(next(state.glob("*.json")).read_text())["scan_offset"] == 0


def test_native_state_storage_failure_keeps_resume_context(worker: Path, tmp_path: Path) -> None:
    occupied = tmp_path / "file"
    occupied.write_text("occupied", encoding="utf-8")
    output = native(worker, {"hook_event_name": "SessionStart", "cwd": str(ROOT)}, occupied)
    assert output is not None
    assert "Ledger/State.md" in output["hookSpecificOutput"]["additionalContext"]


def test_native_git_commit_guard(worker: Path, tmp_path: Path) -> None:
    subprocess.run(["git", "init", "-q", "-b", "master"], cwd=tmp_path, check=True)
    result = subprocess.run([str(worker), "guard-commit", "--root", str(tmp_path)], check=False)
    assert result.returncode == 1
    subprocess.run(["git", "switch", "-c", "feature/example"], cwd=tmp_path, check=True)
    assert (
        subprocess.run(
            [str(worker), "guard-commit", "--root", str(tmp_path)], check=False
        ).returncode
        == 0
    )
