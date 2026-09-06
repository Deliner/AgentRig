use super::resolve;
use serde_json::json;
use std::{fs, path::Path};

fn put(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

fn package(values: &str) -> String {
    format!("schema_version: 1\nid: common\nversion: '1.2'\nconfiguration:\n{values}")
}

#[test]
fn external_package_is_reusable_and_values_retain_origins() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    put(
        root,
        "shared/package.yaml",
        &package("  commands:\n    test:\n      argv: [python3, -m, pytest]\n"),
    );
    put(
        root,
        "first/agentrig.yaml",
        "packages:\n- path: ../shared/package.yaml\n  id: common\n  version: '1.2'\nversion: 1\n",
    );
    put(
        root,
        "second/agentrig.yaml",
        "packages:\n- path: ../shared/package.yaml\noverrides: [/commands/test/argv]\ncommands:\n  test:\n    argv: [cargo, test]\n",
    );
    let first = resolve(&root.join("first/agentrig.yaml")).unwrap();
    let second = resolve(&root.join("second/agentrig.yaml")).unwrap();
    assert_eq!(
        first.configuration["commands"]["test"]["argv"],
        json!(["python3", "-m", "pytest"])
    );
    assert_eq!(
        second.configuration["commands"]["test"]["argv"],
        json!(["cargo", "test"])
    );
    assert_eq!(
        first.provenance["/commands/test/argv"],
        root.join("shared/package.yaml")
    );
    assert_eq!(
        second.provenance["/commands/test/argv"],
        root.join("second/agentrig.yaml")
    );
    assert_eq!(first.packages[0].digest, second.packages[0].digest);
    assert_eq!(first.packages[0].version, "1.2");
}

#[test]
fn repeated_package_is_applied_once_and_exact_versions_are_checked() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    put(root, "common.yaml", &package("  enabled: true\n"));
    let source = "packages:\n- path: common.yaml\n- path: ./common.yaml\n  version: '1.2'\n";
    put(root, "root.yaml", source);
    let resolved = resolve(&root.join("root.yaml")).unwrap();
    assert_eq!(resolved.packages.len(), 1);
    assert_eq!(resolved.configuration["enabled"], true);
    put(root, "root.yaml", &source.replace("'1.2'", "'2.0'"));
    let error = resolve(&root.join("root.yaml")).err().unwrap();
    assert!(format!("{error:#}").contains("package version mismatch"));
}

#[test]
fn named_checks_merge_by_id_and_preserve_declared_order() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    put(
        root,
        "common.yaml",
        &package("  checks:\n  - id: lint\n    warning: true\n  - id: tests\n    command: test\n"),
    );
    put(
        root,
        "root.yaml",
        "packages:\n- path: common.yaml\noverrides: [/checks/lint]\nchecks:\n- id: lint\n  warning: false\n- id: docs\n  command: docs\n",
    );
    let resolved = resolve(&root.join("root.yaml")).unwrap();
    assert_eq!(
        resolved.configuration["checks"],
        json!([
            {"id": "lint", "warning": false}, {"id": "tests", "command": "test"},
            {"id": "docs", "command": "docs"}
        ])
    );
    assert_eq!(
        resolved.provenance["/checks/lint/warning"],
        root.join("root.yaml")
    );
    assert_eq!(
        resolved.provenance["/checks/tests/command"],
        root.join("common.yaml")
    );
}

#[test]
fn conflicts_and_unused_overrides_are_actionable() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    put(root, "common.yaml", &package("  enabled: true\n"));
    for (source, expected) in [
        ("enabled: false\n", "configuration conflict at /enabled"),
        ("overrides: [/absent]\n", "unused override"),
        (
            "overrides: [/enabled, /enabled]\nenabled: false\n",
            "duplicate override",
        ),
    ] {
        put(
            root,
            "root.yaml",
            &format!("packages:\n- path: common.yaml\n{source}"),
        );
        let error = resolve(&root.join("root.yaml")).err().unwrap();
        assert!(format!("{error:#}").contains(expected), "{error:#}");
    }
}

#[test]
fn package_cycles_and_identity_conflicts_are_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    put(root, "root.yaml", "packages:\n- path: common.yaml\n");
    let values = package("  enabled: true\n");
    put(
        root,
        "common.yaml",
        &format!("{values}packages:\n- path: common.yaml\n"),
    );
    let error = resolve(&root.join("root.yaml")).err().unwrap();
    assert!(format!("{error:#}").contains("package cycle"));
    put(root, "common.yaml", &values);
    put(root, "copy.yaml", &values);
    put(
        root,
        "root.yaml",
        "packages:\n- path: common.yaml\n- path: copy.yaml\n",
    );
    let error = resolve(&root.join("root.yaml")).err().unwrap();
    assert!(format!("{error:#}").contains("package identity conflict"));
}

#[test]
fn strict_yaml_and_package_schema_reject_unsupported_inputs() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    put(root, "root.yaml", "packages:\n- path: common.yaml\n");
    for (source, expected) in [
        (
            "schema_version: 2\nid: common\nversion: '1'\nconfiguration: {}\n",
            "unsupported package schema_version",
        ),
        (
            "schema_version: 1\nid: common\nversion: 1\nconfiguration: {}\n",
            "string",
        ),
        (
            "schema_version: 1\nid: common\nversion: '1'\nconfiguration: {}\nunknown: true\n",
            "unknown field",
        ),
        (
            "schema_version: 1\nid: common\nversion: '1'\nconfiguration: &a {}\n",
            "anchors",
        ),
        (
            "schema_version: 1\nid: common\nversion: '1'\nconfiguration: {x: 1, x: 2}\n",
            "duplicate",
        ),
    ] {
        put(root, "common.yaml", source);
        let error = resolve(&root.join("root.yaml")).err().unwrap();
        assert!(format!("{error:#}").contains(expected), "{error:#}");
    }
}

#[test]
fn whole_list_override_disables_checks_and_updates_digest() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    put(
        root,
        "common.yaml",
        &package("  checks:\n  - id: tests\n    command: test\n"),
    );
    put(
        root,
        "root.yaml",
        "packages:\n- path: common.yaml\noverrides: [/checks]\nchecks: []\n",
    );
    let first = resolve(&root.join("root.yaml")).unwrap();
    assert_eq!(first.configuration["checks"], json!([]));
    assert!(!first.provenance.contains_key("/checks/tests/command"));
    put(
        root,
        "common.yaml",
        &package("  checks:\n  - id: tests\n    command: changed\n"),
    );
    let second = resolve(&root.join("root.yaml")).unwrap();
    assert_ne!(first.packages[0].digest, second.packages[0].digest);
}

#[test]
fn duplicate_named_entries_are_rejected_even_without_an_import() {
    let directory = tempfile::tempdir().unwrap();
    put(
        directory.path(),
        "root.yaml",
        "rules:\n- id: size\n- id: size\n",
    );
    let error = resolve(&directory.path().join("root.yaml")).err().unwrap();
    assert!(format!("{error:#}").contains("duplicate ID size at /rules"));
}

#[test]
fn transitive_packages_resolve_from_their_own_directories() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    transitive_fixture(root);
    let first = resolve(&root.join("consumer.yaml")).unwrap();
    let second = resolve(&root.join("consumer.yaml")).unwrap();
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&second).unwrap()
    );
    assert_eq!(
        first
            .packages
            .iter()
            .map(|package| package.id.as_str())
            .collect::<Vec<_>>(),
        ["common", "special"]
    );
    assert_eq!(
        first.provenance["/commands/test/argv"],
        root.join("shared/common.yaml")
    );
    assert_eq!(
        first.provenance["/commands/docs/argv"],
        root.join("special/package.yaml")
    );
    assert!(!first.provenance.contains_key("/commands"));
}

fn transitive_fixture(root: &Path) {
    put(
        root,
        "shared/common.yaml",
        &package("  commands:\n    test:\n      argv: [test]\n"),
    );
    put(
        root,
        "special/package.yaml",
        "schema_version: 1\nid: special\nversion: '1'\npackages:\n- path: ../shared/common.yaml\nconfiguration:\n  commands:\n    docs:\n      argv: [docs]\n",
    );
    put(
        root,
        "consumer.yaml",
        "packages:\n- path: special/package.yaml\n- path: shared/common.yaml\n",
    );
}
