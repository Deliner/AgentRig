use super::{gated, large, stage_an_oversized_source};
use crate::shared::testing::{FAIL, GATE, REPAIR, invoke, project, rerun};
use std::fs;

const CANDIDATE: &str = "- id: candidate-lint
  kind: command
  command: candidate-lint
  skill: guides/repair/SKILL.md
- id: lint
  kind: lint
  skill: guides/repair/SKILL.md
";

// DECISION: D217
#[test]
fn the_installed_lint_blocks_code_that_a_weakened_candidate_accepts() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let weakened = "candidate-lint:\n  argv: [sh, candidate.sh]\n";
    project(&root, weakened, CANDIDATE);
    fs::write(root.join("candidate.sh"), "exit 0\n").unwrap();
    stage_an_oversized_source(&root);
    let result = invoke(&root, &["check", "--staged"]);
    assert_eq!(result.code, Some(1), "{}", result.output());
    let accepted = result.stdout.contains("PASS [candidate-lint]");
    let refused = result.stderr.contains("ERROR [lint]");
    assert!(accepted && refused, "{}", result.output());
}

#[test]
fn gate_warns_on_a_failing_warning_check_and_blocks_on_an_error() {
    let (_temp, root) = gated();
    let warned = invoke(&root, &["check"]);
    assert_eq!(warned.code, Some(0), "{}", warned.output());
    let guided = warned.stderr.contains("WARNING [tests]") && warned.stderr.contains(REPAIR);
    assert!(guided, "{}", warned.stderr);
    large(&root, 61);
    let blocked = invoke(&root, &["check"]);
    assert_eq!(blocked.code, Some(1), "{}", blocked.output());
    let stopped =
        blocked.stderr.contains("ERROR [lint]") && !blocked.stderr.contains("WARNING [tests]");
    assert!(stopped, "{}", blocked.stderr);
}

#[test]
fn missing_check_command_is_actionable() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let missing = FAIL.replace("[sh, -c, exit 23]", "[missing-worker-test-executable]");
    project(&root, &missing, GATE);
    let result = invoke(&root, &["check"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    let actionable = result.stderr.contains("cannot execute") && result.stderr.contains(REPAIR);
    assert!(actionable, "{}", result.stderr);
}

#[test]
fn targeted_rerun_keeps_warning_policy() {
    let (_temp, root) = gated();
    let result = invoke(&root, &["check", "--only", "tests"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    assert!(!result.stdout.contains("PASS [lint]"), "{}", result.stdout);
    let warned =
        result.stderr.contains("WARNING [tests]") && result.stderr.contains("**: exited 23");
    assert!(warned, "{}", result.stderr);
    let repeated = rerun(&result.stderr);
    assert_eq!(repeated.code, result.code, "{}", repeated.output());
    assert_eq!(repeated.stderr, result.stderr);
}

#[test]
fn invalid_check_selection_runs_no_check() {
    let (_temp, root) = gated();
    let selections: [&[&str]; 4] = [
        &["check", "--only"],
        &["check", "--only", "unknown"],
        &["check", "--staged", "--staged"],
        &["check", "--only", "lint", "--only", "tests"],
    ];
    for selection in selections {
        let result = invoke(&root, selection);
        assert_eq!(result.code, Some(2), "{selection:?}: {}", result.output());
        assert!(
            !result.stdout.contains("PASS ["),
            "{selection:?}: {}",
            result.stdout
        );
    }
}
