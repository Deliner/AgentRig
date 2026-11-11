use super::consumer::{append, committed};
use crate::shared::testing::{git, invoke};
use std::{fs, path::Path};

const EMPTY_INDEX: &str = "# Decisions\n\n| ID | Decision | Applies in |\n";

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
