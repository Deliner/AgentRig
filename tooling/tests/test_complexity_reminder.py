from pathlib import Path

from complexity_discipline_reminder import configuration

# DECISION: D006

ROOT = Path(__file__).parents[2]


# INVARIANT: I007
def test_complexity_schedule_is_loaded() -> None:
    path = ROOT / ".agents" / "skills" / "complexity-discipline" / "context-reminder.json"
    attention, full, _attention_message, full_message = configuration(path)
    assert attention == 35_000
    assert full == 140_000
    assert "FULL_REFRESH_REQUIRED" in full_message
