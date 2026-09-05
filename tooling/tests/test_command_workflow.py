from pathlib import Path

from check_commands import validate
from native_support import dispatch

# DECISION: D005

ROOT = Path(__file__).parents[2]


# INVARIANT: I005
def test_shell_guard_accepts_only_catalogued_just() -> None:
    for command in ["git status", "just status && git reset --hard", "just unknown"]:
        result = dispatch(
            {
                "hook_event_name": "PreToolUse",
                "tool_name": "Bash",
                "tool_input": {"command": command},
            }
        )
        assert result is not None
        assert result["hookSpecificOutput"]["permissionDecision"] == "deny"
    assert (
        dispatch(
            {
                "hook_event_name": "PreToolUse",
                "tool_name": "Bash",
                "tool_input": {"command": "just status"},
            }
        )
        is None
    )
    assert validate(ROOT) == []
