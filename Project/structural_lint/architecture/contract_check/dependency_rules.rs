use super::diagnostic::diagnostic;
use crate::shared::diagnostics::Diagnostic;
use crate::structural_lint::architecture::contract_rules::{
    self, Boundaries, Dependency, Issue, own_uses,
};
use crate::structural_lint::rule_set::lint_config::Rule;
use crate::structural_lint::targeting::{file_discovery::Inventory, lint_targets};
use anyhow::Result;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

// DECISION: D225
// DECISION: D228
#[derive(Default)]
pub struct Edges {
    dependencies: Vec<Dependency>,
    groups: BTreeSet<PathBuf>,
}

impl Edges {
    pub(super) fn new(dependencies: Vec<Dependency>, groups: BTreeSet<PathBuf>) -> Self {
        Self {
            dependencies,
            groups,
        }
    }
    pub fn append(&mut self, mut more: Edges) {
        self.dependencies.append(&mut more.dependencies);
        self.groups.append(&mut more.groups);
    }
}

// DECISION: D225
pub fn file_cycles(rule: &Rule, edges: &Edges, inventory: &Inventory) -> Result<Vec<Diagnostic>> {
    let directories = directories(rule, inventory)?;
    let cycles = contract_rules::file_cycles(&edges.dependencies, &directories);
    findings(rule, cycles)
}

// DECISION: D227
pub fn entry_uses(rule: &Rule, edges: &Edges, inventory: &Inventory) -> Result<Vec<Diagnostic>> {
    let directories = directories(rule, inventory)?;
    findings(rule, own_uses(&edges.dependencies, &directories))
}

// DECISION: D228
pub fn entry_bypasses(
    rule: &Rule,
    edges: &Edges,
    inventory: &Inventory,
) -> Result<Vec<Diagnostic>> {
    let directories = directories(rule, inventory)?;
    let boundaries = Boundaries::new(&edges.groups, &inventory.files);
    findings(rule, boundaries.bypasses(&edges.dependencies, &directories))
}

fn directories<'i>(rule: &Rule, inventory: &'i Inventory) -> Result<BTreeSet<&'i Path>> {
    let selected = lint_targets::select(rule, inventory)?;
    Ok(selected.iter().map(|target| target.path).collect())
}

fn findings(rule: &Rule, issues: Vec<Issue>) -> Result<Vec<Diagnostic>> {
    let distinct: BTreeSet<Issue> = issues.into_iter().collect();
    distinct
        .into_iter()
        .map(|issue| diagnostic(rule, issue))
        .collect()
}
