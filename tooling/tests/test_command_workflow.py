from pathlib import Path

from check_commands import validate
from just_command_guard import rejection

# DECISION: D005

ROOT = Path(__file__).parents[2]


# INVARIANT: I005
def test_shell_guard_accepts_only_catalogued_just() -> None:
    assert rejection("git status") is not None
    assert rejection("just status && git reset --hard") is not None
    assert rejection("just unknown") is not None
    assert rejection("just status") is None
    assert validate(ROOT) == []
