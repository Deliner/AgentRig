use super::evidence;
use crate::shared::testing::{FAIL, GATE, git, invoke, memory, project, rerun};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

fn repository() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(
        &root,
        FAIL,
        &GATE.replace("warning: true", "warning: false"),
    );
    memory(&root);
    fs::write(root.join(".gitignore"), ".runtime/\n").unwrap();
    fs::write(root.join("src/value.py"), "value = 1\n").unwrap();
    git(&root, &["init", "-q", "-b", "trunk"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "baseline"]);
    (temp, root)
}

fn candidate(root: &Path) -> String {
    git(root, &["add", "-u"]);
    git(root, &["commit", "-qm", "candidate"]);
    git(root, &["rev-parse", "HEAD"])
}

fn replace(path: &Path, from: &str, to: &str) {
    let text = fs::read_to_string(path).unwrap();
    fs::write(path, text.replace(from, to)).unwrap();
}

fn checks(root: &Path) -> Value {
    let resumed = invoke(root, &["resume"]);
    assert_eq!(resumed.code, Some(0), "{}", resumed.output());
    let state: Value = serde_json::from_str(&resumed.stdout).unwrap();
    state["checks"].clone()
}

#[test]
fn revision_gate_uses_exact_files_config_and_repeatable_diagnostics() {
    let (_temp, root) = repository();
    let base = git(&root, &["rev-parse", "HEAD"]);
    let source = root.join("src/value.py");
    fs::write(&source, "line\n".repeat(61)).unwrap();
    let candidate = candidate(&root);
    fs::write(&source, "value = 1\n").unwrap();
    replace(&root.join("lint.yaml"), "error: 60", "error: 100");
    let result = invoke(
        &root,
        &["check", "--revision", &candidate, "--only", "lint"],
    );
    assert_eq!(result.code, Some(1), "{}", result.output());
    assert!(result.stdout.contains("exceeds 60"), "{}", result.stdout);
    let (_, guidance) = result.stderr.split_once("RERUN: ").unwrap();
    let pinned = guidance
        .lines()
        .next()
        .unwrap()
        .contains(&format!("--revision {candidate}"));
    assert!(pinned, "{guidance}");
    assert_eq!(rerun(&result.stderr).code, Some(1));
    let record = evidence(&root);
    assert_eq!(record["revision"], candidate.as_str(), "{record}");
    assert_eq!(record["source"], "revision", "{record}");
    assert_eq!(record["status"], "completed", "{record}");
    assert_eq!(record["code"], 1, "{record}");
    assert_eq!(fs::read_to_string(&source).unwrap(), "value = 1\n");
    let lint = fs::read_to_string(root.join("lint.yaml")).unwrap();
    assert!(lint.contains("error: 100"), "{lint}");
    let based = invoke(&root, &["check", "--revision", &base, "--only", "lint"]);
    assert_eq!(based.code, Some(0), "{}", based.output());
    let checks = checks(&root);
    assert_eq!(checks["last_run"]["revision"], base.as_str(), "{checks}");
    assert_eq!(checks["revision_matches"], false, "{checks}");
    assert_eq!(checks["full_gate_passed"], false, "{checks}");
}

#[test]
fn revision_gate_evidence_distinguishes_failures_success_and_mutated_exports() {
    let (_temp, root) = repository();
    let base = git(&root, &["rev-parse", "HEAD"]);
    let failed = invoke(&root, &["check", "--revision", &base]);
    assert_eq!(failed.code, Some(23), "{}", failed.output());
    assert_eq!(evidence(&root)["code"], 23);
    let commands = root.join(".agentrig/commands.yaml");
    replace(&commands, "exit 23", "exit 0");
    let passing = candidate(&root);
    let passed = invoke(&root, &["check", "--revision", &passing]);
    assert_eq!(passed.code, Some(0), "{}", passed.output());
    assert_eq!(checks(&root)["full_gate_passed"], true);
    replace(&commands, "exit 0", "'echo changed > src/value.py'");
    let changing = candidate(&root);
    let changed = invoke(&root, &["check", "--revision", &changing]);
    assert_eq!(changed.code, Some(2), "{}", changed.output());
    let named = changed.stderr.contains("checked revision inputs changed");
    assert!(named, "{}", changed.stderr);
    assert_eq!(evidence(&root)["status"], "inputs-changed");
    assert_eq!(checks(&root)["full_gate_passed"], false);
    let source = fs::read_to_string(root.join("src/value.py")).unwrap();
    assert_eq!(source, "value = 1\n");
}

#[test]
fn revision_selection_rejects_invalid_or_conflicting_inputs() {
    let (_temp, root) = repository();
    let selections: [&[&str]; 4] = [
        &["check", "--revision"],
        &["check", "--revision", "missing"],
        &["check", "--revision", "HEAD", "--revision", "HEAD"],
        &["check", "--staged", "--revision", "HEAD"],
    ];
    for selection in selections {
        let result = invoke(&root, selection);
        assert_eq!(result.code, Some(2), "{selection:?}: {}", result.output());
        let ran = result.stdout.contains("PASS [");
        let recorded = root.join(".runtime/checks.json").exists();
        assert!(!ran && !recorded, "{selection:?}: {}", result.stdout);
    }
}
