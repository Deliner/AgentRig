from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Any

import pytest
from agent_context import dispatch

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
def test_native_hook_parity(worker: Path, tmp_path: Path, tool: str, payload: Any) -> None:
    event = {
        "hook_event_name": "PreToolUse",
        "tool_name": tool,
        "tool_input": payload,
        "cwd": str(ROOT),
    }
    assert native(worker, event, tmp_path) == dispatch(event)


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


def test_reminder_transcript_and_state_parity(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    transcript = tmp_path / "transcript.jsonl"
    transcript.write_text("", encoding="utf-8")
    old_state, new_state = tmp_path / "old", tmp_path / "new"
    monkeypatch.setenv("COMPLEXITY_DISCIPLINE_STATE_DIR", str(old_state))
    common = {"cwd": str(ROOT), "session_id": "parity", "transcript_path": str(transcript)}
    start = {**common, "hook_event_name": "SessionStart"}
    assert native(worker, start, new_state) == dispatch(start)
    edit = {
        **common,
        "hook_event_name": "PreToolUse",
        "tool_name": "Write",
        "tool_input": {"file_path": "Ledger/State.md"},
    }
    for count in [34_999, 35_000, 35_001, 70_000, 70_001, 140_000, 140_001, 100, 35_100, 35_101]:
        record = {
            "type": "event_msg",
            "payload": {
                "type": "token_count",
                "info": {"last_token_usage": {"input_tokens": count}},
            },
        }
        with transcript.open("a", encoding="utf-8") as output:
            output.write(json.dumps(record) + "\n")
        assert native(worker, edit, new_state) == dispatch(edit)
        old = json.loads(next(old_state.glob("*.json")).read_text())
        new = json.loads(next(new_state.glob("*.json")).read_text())
        assert old == new
    # An unfinished record must remain available for the next scan.
    with transcript.open("a", encoding="utf-8") as output:
        output.write('{"type":')
    assert native(worker, edit, new_state) == dispatch(edit)
    transcript.write_text("", encoding="utf-8")
    assert native(worker, edit, new_state) == dispatch(edit)


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
