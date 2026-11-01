use super::{Contracts, Dependency, EnvironmentUse, Issue};
use std::{
    collections::{BTreeMap, BTreeSet},
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

fn features(contracts: &[(&str, &str)], root: Option<&str>, edges: &[Dependency]) -> Vec<String> {
    let temp = tempfile::tempdir().unwrap();
    for (directory, extra) in contracts {
        write(temp.path(), directory, &format!("purpose: Test\n{extra}"));
    }
    let exclusions = crate::shared::paths::patterns::globs(&[]).unwrap();
    let inventory = crate::structural_lint::inventory::collect(temp.path(), &exclusions).unwrap();
    let directories = contracts.iter().map(|(path, _)| path.into()).collect();
    let (loaded, issues) = Contracts::load(temp.path(), &directories);
    assert!(issues.is_empty(), "{:?}", messages(&issues));
    let found = loaded.check_features(root.map(Path::new), &inventory, edges);
    found
        .iter()
        .map(|issue| format!("{}: {}", issue.path.display(), issue.message))
        .collect()
}

#[test]
fn feature_classification_requires_types_only_on_features_without_nested_features() {
    let contracts = [
        ("Project", ""),
        ("Project/alpha", ""),
        ("Project/alpha/one", "type: flat"),
        ("Project/alpha/tests", ""),
        ("Project/beta", ""),
        ("Project/beta/tests", ""),
        ("Project/gamma", "type: custom"),
        ("Project/gamma/inner", "type: flat"),
        ("outside", "type: flat"),
    ];
    let misplaced = "type belongs only to a feature without nested features under the feature root";
    assert_eq!(
        features(&contracts, Some("Project"), &[]),
        [
            "Project/beta/architecture.yaml: feature without nested features must declare type: flat, layered, hexagonal, onion, clean or custom".into(),
            format!("Project/gamma/inner/architecture.yaml: {misplaced}"),
            format!("outside/architecture.yaml: {misplaced}"),
        ]
    );
    let unrooted = features(&contracts, None, &[]);
    assert_eq!(unrooted.len(), 4, "{unrooted:?}");
    assert!(unrooted.iter().all(|issue| issue.ends_with(misplaced)));
}

#[test]
fn typed_features_require_exactly_their_parts_and_optional_tests() {
    let contracts = [
        ("Project", ""),
        ("Project/core", "type: hexagonal"),
        ("Project/core/adapters", ""),
        ("Project/core/domain", ""),
        ("Project/core/extra", ""),
        ("Project/core/tests", ""),
        ("Project/plain", "type: flat"),
        ("Project/plain/sub", ""),
        ("Project/free", "type: custom"),
        ("Project/free/any", ""),
        ("Project/ok", "type: onion"),
        ("Project/ok/infrastructure", ""),
        ("Project/ok/application", ""),
        ("Project/ok/domain", ""),
    ];
    assert_eq!(
        features(&contracts, Some("Project"), &[]),
        [
            "Project/core/architecture.yaml: hexagonal feature lacks part directory: ports",
            "Project/core/architecture.yaml: hexagonal feature has a directory outside its parts: extra",
            "Project/plain/architecture.yaml: flat feature has a directory outside its parts: sub",
        ]
    );
}

#[test]
fn typed_feature_dependencies_point_inward_and_never_reach_tests() {
    let contracts = [
        ("Project", ""),
        ("Project/core", "type: hexagonal"),
        ("Project/core/adapters", ""),
        ("Project/core/ports", ""),
        ("Project/core/domain", ""),
        ("Project/plain", "type: flat"),
        ("Project/plain/tests", ""),
    ];
    let core = |path: &str| format!("Project/core/{path}");
    let allowed = [
        edge(&core("adapters/a.rs"), &core("domain/d.rs")),
        edge(&core("adapters/a.rs"), &core("ports/p.rs")),
        edge(&core("domain/inner/x.rs"), &core("domain/d.rs")),
        edge(&core("mod.rs"), &core("domain/d.rs")),
        edge(&core("tests/t.py"), &core("domain/d.rs")),
        edge(&core("domain/d.rs"), "Project/other/x.rs"),
        edge("Project/plain/x.py", "Project/plain/tests/t.py"),
        Dependency {
            module_declaration: true,
            ..edge(&core("mod.rs"), &core("tests/t.rs"))
        },
    ];
    assert!(features(&contracts, Some("Project"), &allowed).is_empty());
    let forbidden = [
        edge(&core("domain/d.rs"), &core("adapters/a.rs")),
        edge(&core("ports/p.rs"), &core("mod.rs")),
        edge(&core("mod.rs"), &core("tests/t.py")),
    ];
    let against = "goes against the dependency direction";
    assert_eq!(
        features(&contracts, Some("Project"), &forbidden),
        [
            format!("Project/core/domain/d.rs: hexagonal: domain -> adapters {against}"),
            format!("Project/core/ports/p.rs: hexagonal: ports -> feature root files {against}"),
            format!("Project/core/mod.rs: hexagonal: feature root files -> tests {against}"),
        ]
    );
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
        "purpose: A\ntype: solid",
        "purpose: TODO",
        "purpose: Module",
        "purpose: '...'",
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

// DECISION: D070
#[test]
fn contracts_accept_a_real_purpose_containing_a_generic_word() {
    let root = tempfile::tempdir().unwrap();
    let directories = BTreeSet::from([PathBuf::from("src")]);
    write(root.path(), "src", "purpose: Module loader for the parser");
    assert!(Contracts::load(root.path(), &directories).1.is_empty());
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

// DECISION: D078
#[test]
fn environment_namespaces_match_only_at_a_path_boundary() {
    for external in [
        "std::fs",
        "std::fs::read",
        "::std::env::var",
        "std::process::Command",
    ] {
        assert!(super::environment(external), "{external}");
    }
    for external in [
        "std::collections::BTreeMap",
        "std::path::PathBuf",
        "std::fmt",
        "std::env_logger",
        "fs::read",
        "crate::fs::read",
    ] {
        assert!(!super::environment(external), "{external}");
    }
}

// DECISION: D078
#[test]
fn zone_pure_directories_reject_environment_access_in_their_own_files() {
    let temp = tempfile::tempdir().unwrap();
    write(
        temp.path(),
        "Project/calc",
        "purpose: Pure numeric evaluation\nzone: pure",
    );
    write(temp.path(), "Project/io", "purpose: Filesystem boundary");
    let directories: BTreeSet<_> = ["Project/calc", "Project/io"]
        .into_iter()
        .map(PathBuf::from)
        .collect();
    let (loaded, issues) = Contracts::load(temp.path(), &directories);
    assert!(issues.is_empty(), "{:?}", messages(&issues));
    let use_site = |line, path: &str| EnvironmentUse {
        line,
        path: path.into(),
    };
    let mut environment = BTreeMap::new();
    environment.insert(
        PathBuf::from("Project/calc/math.rs"),
        vec![use_site(3, "std::fs::read"), use_site(9, "std::env::var")],
    );
    environment.insert(
        PathBuf::from("Project/io/sink.rs"),
        vec![use_site(1, "std::fs")],
    );
    let found = loaded.check_zones(&environment);
    let located: Vec<_> = found
        .iter()
        .map(|issue| (issue.path.display().to_string(), issue.line))
        .collect();
    let calc = "Project/calc/math.rs".to_owned();
    assert_eq!(located, [(calc.clone(), Some(3)), (calc, Some(9))]);
    assert!(
        found
            .iter()
            .all(|issue| issue.message.contains("must not access the environment"))
    );
}

// DECISION: D083
#[test]
fn clock_reads_match_only_the_now_constructors() {
    for external in [
        "std::time::Instant::now",
        "std::time::SystemTime::now",
        "::std::time::SystemTime::now",
    ] {
        assert!(super::clock(external), "{external}");
    }
    for external in [
        "std::time::Duration",
        "std::time::Instant",
        "std::time::SystemTime::UNIX_EPOCH",
        "std::time::Duration::from_secs",
    ] {
        assert!(!super::clock(external), "{external}");
    }
}

// DECISION: D083
#[test]
fn zone_pure_directories_reject_wall_clock_reads_in_their_own_files() {
    let temp = tempfile::tempdir().unwrap();
    write(
        temp.path(),
        "Project/calc",
        "purpose: Pure numeric evaluation\nzone: pure",
    );
    let directories = BTreeSet::from([PathBuf::from("Project/calc")]);
    let (loaded, issues) = Contracts::load(temp.path(), &directories);
    assert!(issues.is_empty(), "{:?}", messages(&issues));
    let use_site = |line, path: &str| EnvironmentUse {
        line,
        path: path.into(),
    };
    let mut environment = BTreeMap::new();
    environment.insert(
        PathBuf::from("Project/calc/clock.rs"),
        vec![
            use_site(4, "std::time::Instant::now"),
            use_site(8, "std::time::SystemTime::now"),
        ],
    );
    let found = loaded.check_zones(&environment);
    let lines: Vec<_> = found.iter().map(|issue| issue.line).collect();
    assert_eq!(lines, [Some(4), Some(8)]);
    assert!(
        found
            .iter()
            .all(|issue| issue.message.contains("must not read the wall clock"))
    );
}

// DECISION: D079
#[test]
fn coupling_ignores_same_directory_and_module_declaration_edges() {
    let temp = tempfile::tempdir().unwrap();
    write(
        temp.path(),
        "Project/hub",
        "purpose: Shared helpers\ndependents: 1",
    );
    let directories = BTreeSet::from([PathBuf::from("Project/hub")]);
    let (loaded, issues) = Contracts::load(temp.path(), &directories);
    assert!(issues.is_empty(), "{:?}", messages(&issues));
    let hub = |path: &str| format!("Project/hub/{path}");
    let mut edges = vec![
        edge("Project/a/f.rs", &hub("mod.rs")),
        edge("Project/b/f.rs", &hub("mod.rs")),
        edge(&hub("one.rs"), &hub("two.rs")),
    ];
    edges.push(Dependency {
        module_declaration: true,
        ..edge("Project/c/f.rs", &hub("mod.rs"))
    });
    let found = loaded.check_coupling(&edges);
    assert_eq!(found.len(), 1, "{:?}", messages(&found));
    assert_eq!(
        found[0].path,
        PathBuf::from("Project/hub/architecture.yaml")
    );
    assert!(
        found[0]
            .message
            .contains("depended on by 2 other directories (budget 1)")
    );
}

// DECISION: D079
#[test]
fn coupling_uses_the_default_budget_when_unspecified() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "Project/wide", "purpose: Depended on widely");
    let directories = BTreeSet::from([PathBuf::from("Project/wide")]);
    let (loaded, _) = Contracts::load(temp.path(), &directories);
    let into_wide = |count: usize| -> Vec<Dependency> {
        (0..count)
            .map(|n| edge(&format!("Project/d{n}/f.rs"), "Project/wide/f.rs"))
            .collect()
    };
    let over = loaded.check_coupling(&into_wide(7));
    assert_eq!(
        messages(&over),
        [
            "directory is depended on by 7 other directories (budget 6); reduce incoming coupling or raise the declared dependents budget"
        ]
    );
    assert!(loaded.check_coupling(&into_wide(6)).is_empty());
}
