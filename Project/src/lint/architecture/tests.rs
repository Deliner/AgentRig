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

fn contracts(root: &Path, declarations: &[(&str, &str)]) -> Contracts {
    let mut directories = BTreeSet::new();
    for (directory, source) in declarations {
        write(root, directory, source);
        directories.insert(PathBuf::from(directory));
    }
    let (contracts, issues) = Contracts::load(root, &directories);
    assert!(issues.is_empty(), "{issues:?}");
    contracts
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
fn public_api_is_allowed_but_private_file_and_denied_api_are_reported() {
    let root = tempfile::tempdir().unwrap();
    let policies = contracts(
        root.path(),
        &[
            (
                "src/app",
                "purpose: Application\nallow: ['src/domain/**']\ndeny: ['src/domain/admin.py']",
            ),
            ("src/domain", "purpose: Domain\npublic: [api.py, admin.py]"),
        ],
    );
    assert!(
        policies
            .check(&[edge("src/app/main.py", "src/domain/api.py")])
            .is_empty()
    );
    let issues = policies.check(&[edge("src/app/main.py", "src/domain/private.py")]);
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("private access"));
    assert_eq!(issues[0].path, PathBuf::from("src/app/main.py"));
    assert_eq!(issues[0].line, Some(7));
    let issues = policies.check(&[edge("src/app/main.py", "src/domain/admin.py")]);
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("forbidden dependency"));
}

#[test]
fn nested_contract_cannot_open_a_parent_boundary() {
    let root = tempfile::tempdir().unwrap();
    let policies = contracts(
        root.path(),
        &[
            ("src/app", "purpose: App\nallow: ['src/domain/**']"),
            ("src/domain", "purpose: Domain\npublic: [api.py]"),
            (
                "src/domain/internal",
                "purpose: Implementation\npublic: ['**']",
            ),
        ],
    );
    let issues = policies.check(&[edge("src/app/main.py", "src/domain/internal/data.py")]);
    assert_eq!(issues.len(), 1, "{issues:?}");
    assert!(issues[0].message.contains("across src/domain (public)"));
}

#[test]
fn nested_outbound_permissions_must_respect_parent_restrictions() {
    let root = tempfile::tempdir().unwrap();
    let policies = contracts(
        root.path(),
        &[
            ("src/app", "purpose: App\nallow: ['src/domain/**']"),
            ("src/app/io", "purpose: IO\nallow: ['src/**']"),
            ("src/storage", "purpose: Storage\npublic: ['**']"),
        ],
    );
    let issues = policies.check(&[edge("src/app/io/read.rs", "src/storage/lib.rs")]);
    assert_eq!(issues.len(), 1, "{issues:?}");
    assert!(issues[0].message.contains("across src/app (allow/deny)"));
}

#[test]
fn same_directory_and_common_parent_do_not_need_external_permissions() {
    let root = tempfile::tempdir().unwrap();
    let policies = contracts(
        root.path(),
        &[
            (".", "purpose: Project"),
            ("src", "purpose: Source"),
            ("src/a", "purpose: A\nallow: ['src/b/**']"),
            ("src/b", "purpose: B\npublic: [api.ts]"),
        ],
    );
    let edges = [
        edge("src/a/x.ts", "src/a/y.ts"),
        edge("src/a/x.ts", "src/b/api.ts"),
    ];
    assert!(policies.check(&edges).is_empty());
}

#[test]
fn cycles_use_measured_edges_not_permission_declarations() {
    let root = tempfile::tempdir().unwrap();
    let policies = contracts(
        root.path(),
        &[
            ("a", "purpose: A\nallow: ['**']\npublic: ['**']"),
            ("b", "purpose: B\nallow: ['**']\npublic: ['**']"),
            ("c", "purpose: C\nallow: ['**']\npublic: ['**']"),
        ],
    );
    assert!(policies.check(&[]).is_empty());
    let mut edges = vec![edge("a/api.rs", "b/api.py"), edge("b/api.py", "c/api.ts")];
    assert!(policies.check(&edges).is_empty());
    edges.push(edge("c/api.ts", "a/api.rs"));
    let issues = policies.check(&edges);
    assert_eq!(
        messages(&issues),
        ["directory dependency cycle: a -> b -> c -> a"]
    );
    assert_eq!(issues[0].path, PathBuf::from("c/api.ts"));
    assert_eq!(issues[0].line, Some(7));
    edges.reverse();
    assert_eq!(messages(&policies.check(&edges)), messages(&issues));
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
fn cycles_crossing_different_private_subdirectories_are_visible_at_the_boundary() {
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
        "purpose: A\nallow: ['../outside/**']",
        "purpose: A\npublic: ['/absolute']",
        "purpose: A\ndeny: ['[']",
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
