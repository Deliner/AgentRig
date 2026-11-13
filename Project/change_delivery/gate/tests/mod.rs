mod affected_checks;
mod outcomes;
mod revision;
mod staged;

use crate::shared::testing::{FAIL, GATE, git, project};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

fn evidence(root: &Path) -> Value {
    let record = fs::read_to_string(root.join(".runtime/checks.json")).unwrap();
    serde_json::from_str(&record).unwrap()
}

fn gated() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(&root, FAIL, GATE);
    (temp, root)
}

fn large(root: &Path, lines: usize) {
    fs::write(root.join("src/large.py"), "line\n".repeat(lines)).unwrap();
}

fn stage_an_oversized_source(root: &Path) {
    git(root, &["init", "-q"]);
    large(root, 61);
    git(root, &["add", "."]);
    large(root, 1);
}
