from pathlib import Path

import pytest

from tooling.worker.src.scaffold.memory.tests.consumer import memory
from tooling.worker.src.scaffold.testing.consumer import CONFIG, invoke, project

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
def test_state_recovery_contract(
    worker: Path, tmp_path: Path, source: str | None, valid: bool
) -> None:
    ledger = tmp_path / "Ledger"
    project(tmp_path, CONFIG.replace('memory: "notes"', 'memory: "Ledger"'))
    memory(tmp_path).rename(ledger)
    (ledger / "State.md").unlink()
    present = source is not None
    if present:
        assert source is not None
        (ledger / "State.md").write_text(source, encoding="utf-8")
    assert (invoke(worker, tmp_path, "memory-check").returncode == 0) is valid
