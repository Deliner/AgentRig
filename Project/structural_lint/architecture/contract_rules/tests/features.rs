use super::{edge, messages, write};
use crate::structural_lint::architecture::contract_rules::{
    Contracts, Dependency, Loaded, Relation,
};
use crate::structural_lint::targeting::file_discovery;
use std::path::Path;

fn features(contracts: &[(&str, &str)], root: Option<&str>, edges: &[Dependency]) -> Vec<String> {
    let temp = tempfile::tempdir().unwrap();
    for (directory, extra) in contracts {
        write(temp.path(), directory, &format!("purpose: Test\n{extra}"));
    }
    let exclusions = crate::shared::paths::globs(&[]).unwrap();
    let inventory = file_discovery::maintained_files(temp.path(), &exclusions).unwrap();
    let directories = contracts.iter().map(|(path, _)| path.into()).collect();
    let Loaded {
        contracts: loaded,
        issues,
    } = Contracts::load(temp.path(), &directories);
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
            relation: Relation::Declaration,
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
