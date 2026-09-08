import json
from pathlib import Path

import pytest

from tooling.worker.src.scaffold.testing.consumer import invoke, memory, project

ROW = "| [B001](Backlog/001.md) | Explain slow checks | Locate avoidable delays |\n"
CARD = (
    "# B001 — Explain slow checks\n\n"
    "## Context\n\nA check appeared slow during unrelated feature work.\n\n"
    "## Proposal\n\nShow which check consumes the time.\n\n"
    "## Expected benefit\n\nHelp users identify avoidable delays; benefit is unverified.\n"
)


def write_idea(notes: Path) -> Path:
    index = notes / "Backlog.md"
    existing_index = index.exists()
    initial = (
        index.read_text()
        if existing_index
        else ("# Backlog\n\n| ID | Idea | Expected benefit |\n| --- | --- | --- |\n")
    )
    index.write_text(initial + ROW)
    card = notes / "Backlog/001.md"
    card.parent.mkdir(exist_ok=True)
    card.write_text(CARD)
    return card


@pytest.mark.parametrize("frontend", ["codex", "claude-code"])
def test_backlog_delivery_and_preservation(worker: Path, tmp_path: Path, frontend: str) -> None:
    result = invoke(
        worker,
        tmp_path,
        "init",
        "--frontend",
        frontend,
        "--memory",
        "ideas",
        "--skills",
        "guides",
    )
    assert result.returncode == 0, result.stderr
    notes = tmp_path / "ideas"
    assert (notes / "Backlog.md").is_file()
    assert invoke(worker, tmp_path, "memory-check").returncode == 0
    skill = tmp_path / "guides/edit-backlog/SKILL.md"
    assert skill.is_file()
    plan = (notes / "Plan.md").read_bytes()
    card = write_idea(notes)
    assert invoke(worker, tmp_path, "memory-check").returncode == 0
    event = dict(
        hook_event_name="PreToolUse",
        tool_name="Write",
        tool_input={"file_path": str(card)},
        cwd=str(tmp_path),
    )
    output = invoke(worker, tmp_path, "hook", input=json.dumps(event))
    assert output.returncode == 0, output.stderr
    guidance = json.loads(output.stdout)["hookSpecificOutput"]
    assert str(skill) in guidance["additionalContext"]
    assert "permissionDecision" not in guidance
    before = {path: path.read_bytes() for path in [notes / "Backlog.md", card, skill]}
    result = invoke(worker, tmp_path, "setup")
    assert result.returncode == 0, result.stderr
    assert all(path.read_bytes() == content for path, content in before.items())
    assert (notes / "Plan.md").read_bytes() == plan
    assert invoke(worker, tmp_path, "memory-check").returncode == 0


@pytest.mark.parametrize(
    "case",
    [
        ("Backlog.md", ROW, ROW + ROW, "duplicate ID"),
        ("Backlog.md", "Backlog/001.md", "Backlog/002.md", "detail path"),
        ("Backlog.md", "B001", "P001", "invalid or duplicate ID"),
        ("Backlog/001.md", "## Proposal", "## Unexpected", "sections must be"),
        ("Backlog/001.md", "Show which check consumes the time.", "", "empty section"),
    ],
)
def test_backlog_rejects_broken_records(
    worker: Path, tmp_path: Path, case: tuple[str, str, str, str]
) -> None:
    relative, old, new, diagnostic = case
    project(tmp_path)
    notes = memory(tmp_path)
    write_idea(notes)
    path = notes / relative
    original = path.read_text()
    path.write_text(original.replace(old, new))
    result = invoke(worker, tmp_path, "memory-check")
    assert result.returncode == 2 and diagnostic in result.stderr
    path.write_text(original)
    assert invoke(worker, tmp_path, "memory-check").returncode == 0


@pytest.mark.parametrize("missing", ["card", "row", "index"])
def test_backlog_reports_lost_links(worker: Path, tmp_path: Path, missing: str) -> None:
    project(tmp_path)
    notes = memory(tmp_path)
    # A pre-Backlog consumer remains valid without an index or any idea cards.
    assert invoke(worker, tmp_path, "memory-check").returncode == 0
    card = write_idea(notes)
    index = notes / "Backlog.md"
    missing_card = missing == "card"
    missing_row = missing == "row"
    if missing_card:
        card.unlink()
    elif missing_row:
        index.write_text(index.read_text().replace(ROW, ""))
    else:
        index.unlink()
    result = invoke(worker, tmp_path, "memory-check")
    expected = "missing or escaping memory link" if missing_card else "unindexed detail"
    assert result.returncode == 2 and expected in result.stderr
