use crate::shared::testing::{
    FAIL, GATE, checked_repository, git, git_run, invoke, memory, project, snapshot,
};
use serde_json::Value;
use std::{fs, path::Path};

fn resumed(root: &Path) -> Value {
    let result = invoke(root, &["resume"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    serde_json::from_str(&result.stdout).unwrap()
}

fn null(value: &Value, key: &str) -> bool {
    value.get(key) == Some(&Value::Null)
}

fn record(root: &Path, line: &str) {
    let state = root.join("notes/State.md");
    let text = fs::read_to_string(&state).unwrap();
    fs::write(&state, format!("{text}\n{line}\n")).unwrap();
}

#[test]
fn resume_resolves_state_revision() {
    let (_temp, root) = checked_repository();
    let saved = git(&root, &["rev-parse", "HEAD"]);
    record(
        &root,
        &format!("Branch: `trunk`\nRevision: `{}`", &saved[..12]),
    );
    let value = resumed(&root);
    assert_eq!(value["snapshot"], "current", "{value}");
    assert_eq!(value["state_revision"]["head_changed"], false);
    assert_eq!(value["vcs"]["revision"], saved.as_str());
    assert_eq!(value["vcs"]["branch"], "trunk");
    assert!(null(&value["vcs"], "operation"), "{value}");
    git(&root, &["commit", "-qam", "candidate"]);
    let value = resumed(&root);
    assert_eq!(value["snapshot"], "stale", "{value}");
    assert_eq!(value["state_revision"]["head_changed"], true);
}

#[test]
fn resume_does_not_verify_branch_only_state() {
    let (_temp, root) = checked_repository();
    let initial = resumed(&root);
    let branch = initial["vcs"]["branch"].as_str().unwrap();
    record(&root, &format!("Branch: {branch}"));
    assert_eq!(resumed(&root)["snapshot"], "unverified");
    git(&root, &["commit", "-qam", "candidate"]);
    let result = resumed(&root);
    assert_ne!(result["vcs"]["revision"], initial["vcs"]["revision"]);
    assert_eq!(result["snapshot"], "unverified", "{result}");
    assert!(null(&result["state_revision"], "head_changed"), "{result}");
    let state = root.join("notes/State.md");
    let text = fs::read_to_string(&state).unwrap();
    fs::write(&state, text.replace("Branch: ", "Branch: another-")).unwrap();
    assert_eq!(resumed(&root)["snapshot"], "stale");
}

// INVARIANT: I019
#[test]
fn resume_observes_real_conflict() {
    for operation in ["merge", "rebase"] {
        let (_temp, root) = checked_repository();
        let source = root.join("src/value.py");
        git(&root, &["switch", "-qc", "task/conflict"]);
        fs::write(&source, "value = 2\n").unwrap();
        git(&root, &["commit", "-qam", "feature"]);
        git(&root, &["switch", "-q", "trunk"]);
        fs::write(&source, "value = 3\n").unwrap();
        git(&root, &["commit", "-qam", "base"]);
        git(&root, &["switch", "-q", "task/conflict"]);
        let failed = git_run(&root, &[operation, "trunk"]);
        assert_ne!(failed.code, Some(0), "{operation}: {}", failed.output());
        let before = snapshot(&root);
        assert_eq!(resumed(&root)["vcs"]["operation"], operation);
        assert_eq!(snapshot(&root), before, "{operation}");
    }
}

#[test]
fn resume_distinguishes_plain_and_unborn_repositories() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(&root, FAIL, GATE);
    memory(&root);
    let plain = resumed(&root);
    assert!(null(&plain, "vcs"), "{plain}");
    git(&root, &["init", "-q"]);
    let before = snapshot(&root);
    let value = resumed(&root);
    assert_eq!(value["vcs"]["revision"], "", "{value}");
    assert!(null(&value["vcs"], "operation"), "{value}");
    assert_eq!(snapshot(&root), before);
}

#[test]
fn resume_observes_linked_git_worktree() {
    let (_temp, root) = checked_repository();
    let other = tempfile::tempdir().unwrap();
    let linked = other.path().canonicalize().unwrap().join("linked");
    let path = linked.to_str().unwrap();
    git(&root, &["worktree", "add", "-q", "-b", "task/linked", path]);
    let before = snapshot(&linked);
    assert_eq!(resumed(&linked)["vcs"]["branch"], "task/linked");
    assert_eq!(snapshot(&linked), before);
}
