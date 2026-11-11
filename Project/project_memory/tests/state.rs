// DECISION: D013
use super::consumer::consumer;
use crate::shared::testing::invoke;
use std::fs;

const SNAPSHOT: &str = "# State

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
";

fn checked(source: Option<&str>) -> Option<i32> {
    let (_temp, root) = consumer();
    let state = root.join("notes/State.md");
    fs::remove_file(&state).unwrap();
    if let Some(text) = source {
        fs::write(&state, text).unwrap();
    }
    let result = invoke(&root, &["memory-check"]);
    result.code
}

// INVARIANT: I011
#[test]
fn state_recovery_contract() {
    assert_eq!(checked(Some(SNAPSHOT)), Some(0));
    let (truncated, _) = SNAPSHOT.split_once("## Next action").unwrap();
    let rejected = [
        None,
        Some(SNAPSHOT.replace("## Focus", "## Task")),
        Some(SNAPSHOT.replace("## Progress", "## Workspace")),
        Some(truncated.to_owned()),
        Some(SNAPSHOT.replace("Resume the authorized feature.", "")),
        Some(format!("{SNAPSHOT}\n## Extra\n\nDuplicate history.\n")),
    ];
    for source in rejected {
        assert_eq!(checked(source.as_deref()), Some(2), "{source:?}");
    }
}
