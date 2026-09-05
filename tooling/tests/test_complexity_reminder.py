import json
from pathlib import Path

import pytest
from native_support import dispatch

# DECISION: D006
# DECISION: D019
ROOT = Path(__file__).parents[2]


# INVARIANT: I007
def test_complexity_schedule_is_loaded(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("COMPLEXITY_DISCIPLINE_STATE_DIR", str(tmp_path / "state"))
    config = tmp_path / ".agents/skills/complexity-discipline/context-reminder.json"
    config.parent.mkdir(parents=True)
    config.write_text(
        json.dumps(
            {
                "attention_interval_tokens": 10,
                "full_refresh_interval_tokens": 40,
                "attention_message": "custom attention",
            }
        )
    )
    transcript = tmp_path / "session.jsonl"
    transcript.touch()
    common = {"session_id": "schedule", "transcript_path": str(transcript)}
    dispatch({**common, "hook_event_name": "SessionStart"}, tmp_path)
    edit = {
        **common,
        "hook_event_name": "PreToolUse",
        "tool_name": "Write",
        "tool_input": {"file_path": "source.rs"},
    }
    for tokens, expected in [
        (9, None),
        (10, "custom attention"),
        (11, None),
        (40, "FULL_REFRESH_REQUIRED"),
    ]:
        with transcript.open("a") as stream:
            stream.write(
                json.dumps(
                    {
                        "type": "event_msg",
                        "payload": {
                            "type": "token_count",
                            "info": {"last_token_usage": {"input_tokens": tokens}},
                        },
                    }
                )
                + "\n"
            )
        result = dispatch(edit, tmp_path)
        if expected is None:
            assert result is None
        else:
            assert result is not None
            assert expected in result["hookSpecificOutput"]["permissionDecisionReason"]
