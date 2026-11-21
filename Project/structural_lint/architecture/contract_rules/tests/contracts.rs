use super::write;
use crate::structural_lint::architecture::contract_rules::Contracts;
use std::{collections::BTreeSet, fs, path::PathBuf};

#[test]
fn contracts_reject_missing_invalid_or_escaping_descriptions() {
    let root = tempfile::tempdir().unwrap();
    let directories = BTreeSet::from([PathBuf::from("src")]);
    assert_eq!(Contracts::load(root.path(), &directories).issues.len(), 1);
    for source in [
        "purpose: ''",
        "purpose: Good\nunknown: value",
        "purpose: 123",
        "purpose: A\npurpose: B",
        "purpose: A\ntype: solid",
        "purpose: TODO",
        "purpose: Module",
        "purpose: '...'",
    ] {
        write(root.path(), "src", source);
        let issues = Contracts::load(root.path(), &directories).issues;
        assert_eq!(issues.len(), 1, "{source}");
        assert_eq!(issues[0].path, PathBuf::from("src/architecture.yaml"));
    }
    let outside = BTreeSet::from([PathBuf::from("../outside")]);
    assert!(
        Contracts::load(root.path(), &outside).issues[0]
            .message
            .contains("invalid directory path")
    );
}

// DECISION: D070
#[test]
fn contracts_accept_a_real_purpose_containing_a_generic_word() {
    let root = tempfile::tempdir().unwrap();
    let directories = BTreeSet::from([PathBuf::from("src")]);
    write(root.path(), "src", "purpose: Module loader for the parser");
    assert!(Contracts::load(root.path(), &directories).issues.is_empty());
}

#[test]
fn contracts_reject_symlink_files_and_escaping_parent_directories() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    write(outside.path(), "source", "purpose: Outside");
    let directory = root.path().join("src");
    fs::create_dir(&directory).unwrap();
    symlink(
        outside.path().join("source/architecture.yaml"),
        directory.join("architecture.yaml"),
    )
    .unwrap();
    let directories = BTreeSet::from([PathBuf::from("src")]);
    assert!(
        Contracts::load(root.path(), &directories).issues[0]
            .message
            .contains("regular file")
    );
    fs::remove_file(directory.join("architecture.yaml")).unwrap();
    fs::remove_dir(&directory).unwrap();
    symlink(outside.path().join("source"), &directory).unwrap();
    assert!(
        Contracts::load(root.path(), &directories).issues[0]
            .message
            .contains("escapes project root")
    );
}
