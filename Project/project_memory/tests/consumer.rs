use crate::shared::testing::{FAIL, git, invoke, memory, project};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const MEMORY_CHECK: &str = "- id: memory\n  kind: memory\n  skill: guides/repair/SKILL.md\n";
const HEADINGS: [&str; 5] = ["Context", "Chosen", "Rejected", "Rationale", "Consequences"];

pub fn decision(source: &str) -> String {
    format!("| [D001](Decisions/001.md) | Choice | [code](../{source}) |\n")
}

pub fn append(path: &Path, text: &str) {
    let current = fs::read_to_string(path).unwrap();
    fs::write(path, format!("{current}{text}")).unwrap();
}

pub fn edit(path: &Path, before: &str, after: &str) {
    let text = fs::read_to_string(path).unwrap();
    assert!(text.contains(before), "{before}: {text}");
    fs::write(path, text.replace(before, after)).unwrap();
}

pub fn consumer() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(&root, FAIL, MEMORY_CHECK);
    memory(&root);
    (temp, root)
}

pub fn record(root: &Path, source: &str) {
    let notes = root.join("notes");
    append(&notes.join("Decisions.md"), &decision(source));
    let sections = HEADINGS.map(|heading| format!("## {heading}\n\nText."));
    let detail = format!("# D001\n\n{}", sections.join("\n\n"));
    fs::write(notes.join("Decisions/001.md"), detail).unwrap();
}

pub fn accepts(root: &Path) {
    let checked = invoke(root, &["memory-check"]);
    assert_eq!(checked.code, Some(0), "{}", checked.output());
}

pub fn rejects(root: &Path, args: &[&str], diagnostic: &str) {
    let result = invoke(root, args);
    assert_eq!(result.code, Some(2), "{}", result.output());
    assert!(result.stderr.contains(diagnostic), "{}", result.stderr);
}

pub fn committed() -> (TempDir, PathBuf) {
    let (temp, root) = consumer();
    record(&root, "src/lib.rs");
    let source = "// DECISION: D001\nfn example() {}";
    fs::write(root.join("src/lib.rs"), source).unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "baseline"]);
    accepts(&root);
    (temp, root)
}
