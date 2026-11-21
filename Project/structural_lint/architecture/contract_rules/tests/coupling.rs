use super::{edge, messages, write};
use crate::structural_lint::architecture::contract_rules::{
    Contracts, Dependency, Loaded, Relation,
};
use std::{collections::BTreeSet, path::PathBuf};

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
    let Loaded {
        contracts: loaded,
        issues,
    } = Contracts::load(temp.path(), &directories);
    assert!(issues.is_empty(), "{:?}", messages(&issues));
    let hub = |path: &str| format!("Project/hub/{path}");
    let mut edges = vec![
        edge("Project/a/f.rs", &hub("mod.rs")),
        edge("Project/b/f.rs", &hub("mod.rs")),
        edge(&hub("one.rs"), &hub("two.rs")),
    ];
    edges.push(Dependency {
        relation: Relation::Declaration,
        ..edge("Project/c/f.rs", &hub("mod.rs"))
    });
    let found = loaded.check_coupling(&edges);
    assert_eq!(found.len(), 1, "{:?}", messages(&found));
    assert_eq!(
        found[0].path,
        PathBuf::from("Project/hub/architecture.yaml")
    );
    assert_eq!(
        messages(&found),
        [
            "directory is depended on by 2 other directories (budget 1): Project/a (Project/a/f.rs:7 uses Project/hub/mod.rs), Project/b (Project/b/f.rs:7 uses Project/hub/mod.rs); reduce incoming coupling or raise the declared dependents budget"
        ]
    );
}

// DECISION: D079
#[test]
fn coupling_uses_the_default_budget_when_unspecified() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "Project/wide", "purpose: Depended on widely");
    let directories = BTreeSet::from([PathBuf::from("Project/wide")]);
    let Loaded {
        contracts: loaded, ..
    } = Contracts::load(temp.path(), &directories);
    let into_wide = |count: usize| -> Vec<Dependency> {
        Vec::from_fn(count, |n| {
            edge(&format!("Project/d{n}/f.rs"), "Project/wide/f.rs")
        })
    };
    let over = loaded.check_coupling(&into_wide(7));
    let named = Vec::from_fn(7, |n| {
        format!("Project/d{n} (Project/d{n}/f.rs:7 uses Project/wide/f.rs)")
    });
    assert_eq!(
        messages(&over),
        [format!(
            "directory is depended on by 7 other directories (budget 6): {}; reduce incoming coupling or raise the declared dependents budget",
            named.join(", ")
        )]
    );
    assert!(loaded.check_coupling(&into_wide(6)).is_empty());
}
