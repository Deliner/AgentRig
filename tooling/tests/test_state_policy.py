from pathlib import Path

import pytest
from check_repo import check, state_findings

# DECISION: D013

SNAPSHOT = """# State

## Focus

Resume the authorized feature.

## Workspace

Recorded branch feature/old and revision abc123; reconcile with current Git.

## Progress

An unfinished VAC remains; inspect its diff before continuing.

## Verification

The simulation failed; rerun the recorded focused check after the fix.

## Blockers

None external; the failing check needs correction.

## Next action

Inspect Git and the failed check output, then correct the current VAC.
"""


@pytest.mark.parametrize(
    ("source", "valid"),
    [
        (None, False),
        (SNAPSHOT, True),
        (SNAPSHOT.replace("## Focus", "## Task"), False),
        (SNAPSHOT.replace("## Progress", "## Workspace"), False),
        (SNAPSHOT.split("## Next action")[0], False),
        (SNAPSHOT.replace("Resume the authorized feature.", ""), False),
        (SNAPSHOT + "\n## Extra\n\nDuplicate history.\n", False),
    ],
)
# INVARIANT: I011
def test_state_recovery_contract(tmp_path: Path, source: str | None, valid: bool) -> None:
    ledger = tmp_path / "Ledger"
    ledger.mkdir()
    if source is not None:
        (ledger / "State.md").write_text(source, encoding="utf-8")
    assert (state_findings(tmp_path) == []) is valid


def test_repository_gate_includes_state(tmp_path: Path) -> None:
    (tmp_path / "Project").mkdir()
    (tmp_path / "Project/README.md").write_text("Product", encoding="utf-8")
    ledger = tmp_path / "Ledger"
    ledger.mkdir()
    for name in ("Decisions", "Invariants"):
        (ledger / f"{name}.md").write_text(f"# {name}\n", encoding="utf-8")
    (ledger / "Plan.md").write_text(
        "# Plan\n\n| ID | Status | Depends on | Feature | User capability |\n"
        "| --- | --- | --- | --- | --- |\n",
        encoding="utf-8",
    )
    assert any("Ledger/State.md" in item.message for item in check(tmp_path, tmp_path))
    (ledger / "State.md").write_text(SNAPSHOT, encoding="utf-8")
    assert check(tmp_path, tmp_path) == []
