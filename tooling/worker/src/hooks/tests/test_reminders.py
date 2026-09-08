# DECISION: D019
# DECISION: D006
import json
import subprocess
from pathlib import Path
from typing import Any

import pytest

from tooling.worker.src.scaffold.testing.consumer import invoke, project


@pytest.mark.parametrize("frontend", ["codex", "claude-code"])
# INVARIANT: I007
def test_portable_reminder_schedule_retry_and_compaction(
    worker: Path, tmp_path: Path, frontend: str
) -> None:
    common = configured_session(worker, tmp_path, frontend)
    scratch = tmp_path / ".scratch"
    transcript = scratch / "transcript.jsonl"
    start = {**common, "hook_event_name": "SessionStart"}
    result = invoke(worker, tmp_path, "hook", input=json.dumps(start))
    assert result.returncode == 0
    assert "FULL_REFRESH_REQUIRED" in result.stdout
    assert "guides/complexity-discipline/SKILL.md" in result.stdout
    assert "notes/State.md" in result.stdout
    assert "Ledger/" not in result.stdout and ".agents/" not in result.stdout
    edit = {
        **common,
        "hook_event_name": "PreToolUse",
        "tool_name": "Write",
        "tool_input": {"file_path": "notes/State.md"},
    }
    for count, expected in SCHEDULE:
        append_tokens(transcript, count, frontend)
        result = invoke(worker, tmp_path, "hook", input=json.dumps(edit))
        assert_reminder(result, expected)
    saved = next((scratch / "state/reminders").glob("*.json"))
    assert json.loads(saved.read_text())["scan_offset"] == transcript.stat().st_size
    result = invoke(worker, tmp_path, "hook", input=json.dumps({**start, "source": "compact"}))
    assert "FULL_REFRESH_REQUIRED" in result.stdout
    assert json.loads(saved.read_text())["baseline_tokens"] == 150
    result = invoke(worker, tmp_path, "hook", input=json.dumps(edit))
    assert "permissionDecision" not in json.loads(result.stdout)["hookSpecificOutput"]
    for name in ["Plan", "Decisions", "Invariants", "State"]:
        edit["tool_input"] = {"file_path": f"notes/{name}.md"}
        result = invoke(worker, tmp_path, "hook", input=json.dumps(edit))
        assert f"guides/edit-{name.lower()}/SKILL.md" in result.stdout


def test_no_configured_discipline_does_not_invent_a_skill(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    (tmp_path / ".runtime").write_text("occupied")
    result = invoke(worker, tmp_path, "hook", input=json.dumps({"hook_event_name": "SessionStart"}))
    assert result.returncode == 0
    assert "notes/State.md" in result.stdout
    assert ".agents/" not in result.stdout
    assert "FULL_REFRESH_REQUIRED" not in result.stdout
    assert "Context reminders unavailable" not in result.stdout


@pytest.mark.parametrize("path", ["src/example.rs", "notes/State.md"])
def test_reminder_storage_failure_is_visible_and_recovers(
    worker: Path, tmp_path: Path, path: str
) -> None:
    common = configured_session(worker, tmp_path)
    scratch = tmp_path / ".scratch"
    runtime = scratch / "state"
    runtime.write_text("occupied")
    append_tokens(scratch / "transcript.jsonl", 300)
    event = {
        **common,
        "hook_event_name": "PreToolUse",
        "tool_name": "Edit",
        "tool_input": {"file_path": path},
    }
    result = invoke(worker, tmp_path, "hook", input=json.dumps(event))
    assert result.returncode == 0
    output = json.loads(result.stdout)["hookSpecificOutput"]
    assert "permissionDecision" not in output
    assert "Context reminders unavailable" in output["additionalContext"]
    assert str(runtime / "reminders") in output["additionalContext"]
    assert ("guides/edit-state/SKILL.md" in result.stdout) == (path == "notes/State.md")
    runtime.unlink()
    recovered = invoke(worker, tmp_path, "hook", input=json.dumps(event))
    assert recovered.returncode == 0
    assert "FULL_REFRESH_REQUIRED" in recovered.stdout
    assert "Context reminders unavailable" not in recovered.stdout


@pytest.mark.parametrize(
    "change",
    [
        {"attention_message": 17},
        {"attention_message": ""},
        {"unknown_setting": True},
        {"attention_interval_tokens": 0},
        {"full_refresh_interval_tokens": 100},
    ],
)
def test_reminder_configuration_rejects_malformed_values(
    worker: Path, tmp_path: Path, change: dict[str, object]
) -> None:
    assert invoke(worker, tmp_path, "init").returncode == 0
    path = tmp_path / ".agentrig/reminder.json"
    config = json.loads(path.read_text())
    config.update(change)
    path.write_text(json.dumps(config))
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "hooks.reminder" in result.stderr


def test_partial_and_truncated_transcripts_recover(worker: Path, tmp_path: Path) -> None:
    assert invoke(worker, tmp_path, "init").returncode == 0
    settings = tmp_path / ".agentrig/reminder.json"
    settings.write_text(
        json.dumps({"attention_interval_tokens": 100, "full_refresh_interval_tokens": 300})
    )
    transcript = tmp_path / "transcript.jsonl"
    transcript.touch()
    common = {"session_id": "partial", "transcript_path": str(transcript)}
    invoke(
        worker, tmp_path, "hook", input=json.dumps({**common, "hook_event_name": "SessionStart"})
    )
    event = {
        **common,
        "hook_event_name": "PreToolUse",
        "tool_name": "Edit",
        "tool_input": {"path": "memory/State.md"},
    }
    record = json.dumps(
        {
            "type": "event_msg",
            "payload": {"type": "token_count", "info": {"last_token_usage": {"input_tokens": 100}}},
        }
    )
    transcript.write_text(record[:-2])
    assert "deny" not in invoke(worker, tmp_path, "hook", input=json.dumps(event)).stdout
    saved = next((tmp_path / ".agentrig/runtime/reminders").glob("*.json"))
    assert json.loads(saved.read_text())["scan_offset"] == 0
    transcript.write_text(record + "\n")
    assert "deny" in invoke(worker, tmp_path, "hook", input=json.dumps(event)).stdout
    assert "deny" not in invoke(worker, tmp_path, "hook", input=json.dumps(event)).stdout
    transcript.write_text(record.replace("100", "0") + "\n")
    assert "deny" not in invoke(worker, tmp_path, "hook", input=json.dumps(event)).stdout
    state = json.loads(saved.read_text())
    assert state["scan_offset"] == transcript.stat().st_size
    assert state["latest_tokens"] == 0
    with transcript.open("a") as stream:
        stream.write("invalid complete record\n" + record + "\n")
    assert "deny" in invoke(worker, tmp_path, "hook", input=json.dumps(event)).stdout


SCHEDULE: list[tuple[int, str | None]] = [
    (99, None),
    (100, "REASSESS_CURRENT_WORK"),
    (100, None),
    (200, "REASSESS_CURRENT_WORK"),
    (201, None),
    (300, "FULL_REFRESH_REQUIRED"),
    (301, None),
    (50, None),
    (149, None),
    (150, "REASSESS_CURRENT_WORK"),
]


def configured_session(worker: Path, tmp_path: Path, frontend: str = "codex") -> dict[str, Any]:
    options = ["--memory", "notes", "--skills", "guides", "--frontend", frontend]
    assert invoke(worker, tmp_path, "init", *options).returncode == 0
    config = tmp_path / "agentrig.yaml"
    config.write_text(
        config.read_text()
        .replace("runtime: .agentrig/runtime", "runtime: .scratch/state")
        .replace("reminder: .agentrig/reminder.json", "reminder: .scratch/reminder.json")
    )
    scratch = tmp_path / ".scratch"
    scratch.mkdir()
    (scratch / "reminder.json").write_text(
        json.dumps(
            {
                "attention_interval_tokens": 100,
                "full_refresh_interval_tokens": 300,
                "attention_message": "REASSESS_CURRENT_WORK",
            }
        )
    )
    transcript = scratch / "transcript.jsonl"
    transcript.touch()
    common = {"cwd": str(tmp_path), "session_id": "portable", "transcript_path": str(transcript)}
    return common


def append_tokens(transcript: Path, count: int, frontend: str = "codex") -> None:
    record = {
        "type": "event_msg",
        "payload": {"type": "token_count", "info": {"last_token_usage": {"input_tokens": count}}},
    }
    using_claude = frontend == "claude-code"
    if using_claude:
        # Claude Code 2.1.201 writes these usage fields; output tokens are not context input.
        record = {
            "type": "assistant",
            "message": {
                "usage": {
                    "input_tokens": count // 2,
                    "cache_creation_input_tokens": count // 4,
                    "cache_read_input_tokens": count - count // 2 - count // 4,
                    "output_tokens": 10000,
                }
            },
        }
    with transcript.open("a") as stream:
        stream.write(json.dumps(record) + "\n")


def assert_reminder(result: subprocess.CompletedProcess[str], expected: str | None) -> None:
    assert result.returncode == 0
    output = json.loads(result.stdout)["hookSpecificOutput"]
    no_reminder_expected = expected is None
    if no_reminder_expected:
        assert "permissionDecision" not in output
    else:
        assert output["permissionDecision"] == "deny"
        assert expected is not None and expected in output["permissionDecisionReason"]
    assert "guides/edit-state/SKILL.md" in result.stdout
    assert ".agents/" not in result.stdout
