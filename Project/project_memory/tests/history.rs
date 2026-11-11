use crate::shared::testing::{FAIL, git, invoke, memory, project};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const MEMORY_CHECK: &str = "- id: memory\n  kind: memory\n  skill: guides/repair/SKILL.md\n";
const DECISION: &str = "| [D001](Decisions/001.md) | Choice | [code](../src/lib.rs) |\n";
const EMPTY_INDEX: &str = "# Decisions\n\n| ID | Decision | Applies in |\n";
const HEADINGS: [&str; 5] = ["Context", "Chosen", "Rejected", "Rationale", "Consequences"];

fn append(path: &Path, text: &str) {
    let current = fs::read_to_string(path).unwrap();
    fs::write(path, format!("{current}{text}")).unwrap();
}

fn committed() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(&root, FAIL, MEMORY_CHECK);
    memory(&root);
    let notes = root.join("notes");
    append(&notes.join("Decisions.md"), DECISION);
    let sections = HEADINGS.map(|heading| format!("## {heading}\n\nText."));
    let detail = format!("# D001\n\n{}", sections.join("\n\n"));
    fs::write(notes.join("Decisions/001.md"), detail).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "// DECISION: D001\nfn example() {}",
    )
    .unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "baseline"]);
    let checked = invoke(&root, &["memory-check"]);
    assert_eq!(checked.code, Some(0), "{}", checked.output());
    (temp, root)
}

fn candidate(root: &Path) -> String {
    git(root, &["add", "-u"]);
    git(root, &["commit", "-qm", "candidate"]);
    git(root, &["rev-parse", "HEAD"])
}

fn passes(root: &Path, revision: &str) {
    let result = invoke(root, &["check", "--revision", revision]);
    assert_eq!(result.code, Some(0), "{}", result.output());
}

fn parent_history_rejects(path: &str, damage: fn(&str) -> String) {
    let (_temp, root) = committed();
    let base = git(&root, &["rev-parse", "HEAD"]);
    passes(&root, &base);
    let target = root.join("notes").join(path);
    let original = fs::read_to_string(&target).unwrap();
    fs::write(&target, damage(&original)).unwrap();
    let candidate = candidate(&root);
    fs::write(&target, &original).unwrap();
    let result = invoke(&root, &["check", "--revision", &candidate]);
    assert_eq!(result.code, Some(2), "{}", result.output());
    let named = result.stderr.contains("committed decision");
    assert!(named, "{}", result.stderr);
    assert_eq!(fs::read_to_string(&target).unwrap(), original);
    passes(&root, &base);
}

#[test]
fn revision_memory_rejects_a_changed_identity_from_parent_history() {
    parent_history_rejects("Decisions.md", |text| {
        text.replace("| Choice |", "| Changed |")
    });
}

#[test]
fn revision_memory_rejects_a_changed_detail_from_parent_history() {
    parent_history_rejects("Decisions/001.md", |text| text.replace("Text.", "Changed."));
}

#[test]
fn revision_memory_rejects_a_removed_decision_from_parent_history() {
    parent_history_rejects("Decisions.md", |_| EMPTY_INDEX.to_owned());
}

#[test]
fn revision_memory_checks_the_second_merge_parent() {
    let (_temp, root) = committed();
    let base = git(&root, &["rev-parse", "HEAD"]);
    append(&root.join("src/lib.rs"), "\n// left branch\n");
    let left = candidate(&root);
    git(&root, &["checkout", "-q", &base]);
    let index = root.join("notes/Decisions.md");
    let changed = fs::read_to_string(&index)
        .unwrap()
        .replace("| Choice |", "| Changed |");
    fs::write(&index, changed).unwrap();
    candidate(&root);
    git(&root, &["merge", "--no-commit", &left]);
    let merged = candidate(&root);
    let result = invoke(&root, &["check", "--revision", &merged]);
    assert_eq!(result.code, Some(2), "{}", result.output());
    let named = result
        .stderr
        .contains("committed decision identity cannot change");
    assert!(named, "{}", result.stderr);
}
