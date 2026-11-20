use super::resolve;
use std::{fs, os::unix::fs::symlink};

#[test]
fn missing_tail_is_normalized_without_creating_files() {
    let root = tempfile::tempdir().unwrap();
    let base = root.path().canonicalize().unwrap();
    let path = root.path().join("missing/../new/./leaf");
    assert_eq!(resolve(&path).unwrap(), base.join("new/leaf"));
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn parent_traversal_follows_link_targets_and_absolute_links_replace_the_prefix() {
    let root = tempfile::tempdir().unwrap();
    let base = root.path().canonicalize().unwrap();
    fs::create_dir_all(root.path().join("actual/nested")).unwrap();
    symlink("actual/nested", root.path().join("relative")).unwrap();
    assert_eq!(
        resolve(&root.path().join("relative/../new")).unwrap(),
        base.join("actual/new")
    );
    let outside = tempfile::tempdir().unwrap();
    symlink(outside.path(), root.path().join("absolute")).unwrap();
    assert_eq!(
        resolve(&root.path().join("absolute/new")).unwrap(),
        outside.path().canonicalize().unwrap().join("new")
    );
}

// DECISION: D199
#[test]
fn existing_names_take_the_spelling_of_the_file_system() {
    let root = tempfile::tempdir().unwrap();
    let base = root.path().canonicalize().unwrap();
    fs::create_dir_all(root.path().join("Ledger")).unwrap();
    fs::write(root.path().join("Ledger/State.md"), "").unwrap();
    let written = root.path().join("ledger/STATE.md");
    let same_file = fs::metadata(&written).is_ok();
    let expected = if same_file {
        base.join("Ledger/State.md")
    } else {
        base.join("ledger/STATE.md")
    };
    assert_eq!(resolve(&written).unwrap(), expected);
    let new = root.path().join("ledger/new.md");
    let parent = if same_file { "Ledger" } else { "ledger" };
    assert_eq!(resolve(&new).unwrap(), base.join(parent).join("new.md"));
}

#[test]
fn dangling_links_resolve_but_recursive_links_fail() {
    let root = tempfile::tempdir().unwrap();
    let base = root.path().canonicalize().unwrap();
    symlink("missing", root.path().join("dangling")).unwrap();
    assert_eq!(
        resolve(&root.path().join("dangling/leaf")).unwrap(),
        base.join("missing/leaf")
    );
    symlink("second", root.path().join("first")).unwrap();
    symlink("first", root.path().join("second")).unwrap();
    assert_eq!(
        resolve(&root.path().join("first")).unwrap_err().to_string(),
        "symlink cycle"
    );
}
