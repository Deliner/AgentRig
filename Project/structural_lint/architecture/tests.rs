use super::{Contracts, Dependency, Issue};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

fn write(root: &Path, directory: &str, source: &str) {
    let path = root.join(directory);
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("architecture.yaml"), source).unwrap();
}

fn edge(source: &str, target: &str) -> Dependency {
    Dependency {
        source: source.into(),
        target: target.into(),
        line: 7,
        module_declaration: false,
    }
}

fn messages(issues: &[Issue]) -> Vec<&str> {
    issues.iter().map(|issue| issue.message.as_str()).collect()
}

#[test]
fn cycles_report_the_closing_measured_edge() {
    assert!(super::graph::cycles(&[]).is_empty());
    let mut edges = vec![edge("a/api.rs", "b/api.py"), edge("b/api.py", "c/api.ts")];
    assert!(super::graph::cycles(&edges).is_empty());
    edges.push(edge("c/api.ts", "a/api.rs"));
    let issues = super::graph::cycles(&edges);
    assert_eq!(
        messages(&issues),
        ["directory dependency cycle: a -> b -> c -> a"]
    );
    assert_eq!(issues[0].path, PathBuf::from("c/api.ts"));
    assert_eq!(issues[0].line, Some(7));
    edges.reverse();
    assert_eq!(messages(&super::graph::cycles(&edges)), messages(&issues));
}

#[test]
fn repeated_edges_and_diamond_are_not_cycles() {
    let edges = [
        edge("a/x.rs", "b/x.rs"),
        edge("a/x.rs", "b/x.rs"),
        edge("a/x.rs", "c/x.rs"),
        edge("b/x.rs", "d/x.rs"),
        edge("c/x.rs", "d/x.rs"),
        edge("d/x.rs", "d/y.rs"),
    ];
    assert!(super::graph::cycles(&edges).is_empty());
}

#[test]
fn cycles_crossing_different_subdirectories_are_visible_at_the_boundary() {
    let edges = [
        edge("src/a/impl/read.rs", "src/b/api/entry.rs"),
        edge("src/b/impl/write.rs", "src/a/api/entry.rs"),
    ];
    let issues = super::graph::cycles(&edges);
    assert!(messages(&issues).contains(&"directory dependency cycle: src/a -> src/b -> src/a"));
    assert_eq!(issues[0].line, Some(7));
}

#[test]
fn contracts_reject_missing_invalid_or_escaping_descriptions() {
    let root = tempfile::tempdir().unwrap();
    let directories = BTreeSet::from([PathBuf::from("src")]);
    assert_eq!(Contracts::load(root.path(), &directories).1.len(), 1);
    for source in [
        "purpose: ''",
        "purpose: Good\nunknown: value",
        "purpose: 123",
        "purpose: A\npurpose: B",
    ] {
        write(root.path(), "src", source);
        let issues = Contracts::load(root.path(), &directories).1;
        assert_eq!(issues.len(), 1, "{source}");
        assert_eq!(issues[0].path, PathBuf::from("src/architecture.yaml"));
    }
    let outside = BTreeSet::from([PathBuf::from("../outside")]);
    assert!(
        Contracts::load(root.path(), &outside).1[0]
            .message
            .contains("invalid directory path")
    );
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
        Contracts::load(root.path(), &directories).1[0]
            .message
            .contains("regular file")
    );
    fs::remove_file(directory.join("architecture.yaml")).unwrap();
    fs::remove_dir(&directory).unwrap();
    symlink(outside.path().join("source"), &directory).unwrap();
    assert!(
        Contracts::load(root.path(), &directories).1[0]
            .message
            .contains("escapes project root")
    );
}
