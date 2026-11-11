mod outcomes;
mod selection;
mod staged;

use crate::shared::testing::{FAIL, GATE, git, project};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

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
