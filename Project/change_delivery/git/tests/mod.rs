use crate::shared::testing::{commit, configure, git, invoke, run, runtime};
use std::path::PathBuf;
use tempfile::TempDir;

const PROBE_COMMAND: &str = r#"probe:
  purpose: Block delivery on request
  argv: [sh, -c, 'test -z "$BLOCK_DELIVERY"']
"#;
const PROBE_CHECK: &str = "- id: probe
  kind: command
  command: probe
  skill: .agentrig/skills/repair/SKILL.md
";

fn installed() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    git(&root, &["init", "-q", "-b", "trunk"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    commit(&root, "shared.txt", "initial\n");
    git(&root, &["switch", "-c", "task/bootstrap"]);
    let initialized = invoke(&root, &["init", "--base", "trunk", "--prefix", "task/"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    configure(&root, "commands", PROBE_COMMAND);
    configure(&root, "checks", PROBE_CHECK);
    commit(&root, "ready.txt", "ready");
    let merged = invoke(&root, &["feature-merge"]);
    assert_eq!(merged.code, Some(0), "{}", merged.output());
    (temp, root)
}

// INVARIANT: I021
#[test]
fn reference_guard_rejects_deletion_when_the_tip_cannot_be_resolved() {
    let (_temp, root) = installed();
    let unknown = "0".repeat(40);
    let delete = |branch: &str| {
        let update = format!("{unknown} {unknown} refs/heads/task/{branch}\n");
        run(
            &mut runtime(&root, &["guard-reference", "prepared"]),
            &update,
        )
    };
    let merged = delete("bootstrap");
    assert_eq!(merged.code, Some(1), "{}", merged.output());
    assert!(merged.stderr.contains("retained"), "{}", merged.stderr);
    let missing = delete("missing");
    assert_eq!(missing.code, Some(2), "{}", missing.output());
    assert!(!missing.stderr.contains("retained"), "{}", missing.stderr);
}
