from pathlib import Path

import pytest

from Project.shared.testing.consumer import CONFIG, invoke, memory, project

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
    ("source", "outcome"),
    [
        (None, "rejected"),
        (SNAPSHOT, "accepted"),
        (SNAPSHOT.replace("## Focus", "## Task"), "rejected"),
        (SNAPSHOT.replace("## Progress", "## Workspace"), "rejected"),
        (SNAPSHOT.split("## Next action")[0], "rejected"),
        (SNAPSHOT.replace("Resume the authorized feature.", ""), "rejected"),
        (SNAPSHOT + "\n## Extra\n\nDuplicate history.\n", "rejected"),
    ],
)
# INVARIANT: I011
def test_state_recovery_contract(
    worker: Path, tmp_path: Path, source: str | None, outcome: str
) -> None:
    ledger = tmp_path / "Ledger"
    project(tmp_path, CONFIG.replace('memory: "notes"', 'memory: "Ledger"'))
    memory(tmp_path).rename(ledger)
    (ledger / "State.md").unlink()
    present = source is not None
    if present:
        assert source is not None
        (ledger / "State.md").write_text(source, encoding="utf-8")
    accepted = outcome == "accepted"
    assert (invoke(worker, tmp_path, "memory-check").returncode == 0) is accepted
