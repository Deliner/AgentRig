mod analysis;
mod scope;

use super::{Contracts, Issue, source::References};
use crate::shared::diagnostics::lint::Diagnostic;
use crate::shared::paths::patterns::globs;
use crate::structural_lint::{
    config::Rule,
    inventory::{self, Inventory},
};
use anyhow::Result;
use scope::Scope;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub fn validate(rule: &Rule, inventory: &Inventory) -> Result<()> {
    Scope::new(rule, inventory).map(|_| ())
}

pub fn contains_directory(rule: &Rule, path: &Path, inventory: &Inventory) -> Result<bool> {
    Ok(Scope::new(rule, inventory)?.directories.contains(path))
}

pub fn run(root: &Path, rule: &Rule, inventory: &Inventory) -> Result<Vec<Diagnostic>> {
    let scope = Scope::new(rule, inventory)?;
    let empty = scope.directories.is_empty();
    if empty {
        return Ok(Vec::new());
    }
    let root = root.canonicalize()?;
    let no_exclusions = globs(&[])?;
    let files: BTreeSet<_> = inventory::collect(&root, &no_exclusions)?
        .files
        .into_iter()
        .collect();
    let settings = rule
        .architecture
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("architecture rule without settings"))?;
    let rust_sources = analysis::rust_sources(&root, &files);
    let (resolvers, mut issues) = analysis::Resolvers::new(&root, &files, &rust_sources, settings)?;
    let (contracts, contract_issues) = Contracts::load(&root, &scope.directories);
    issues.extend(contract_issues);
    issues.extend(contracts.check_inventory(&root, &scope.inventory));
    let sources = analysis::extract(&root, &scope.files);
    let (edges, source_issues) = dependencies(&resolvers, sources);
    issues.extend(source_issues);
    issues.extend(super::graph::cycles(&edges));
    let features = settings.feature_root.as_deref();
    issues.extend(contracts.check_features(features, &scope.inventory, &edges));
    issues
        .into_iter()
        .map(|issue| diagnostic(rule, issue))
        .collect()
}

fn dependencies(
    resolvers: &analysis::Resolvers<'_>,
    sources: BTreeMap<PathBuf, References>,
) -> (Vec<super::Dependency>, Vec<Issue>) {
    let mut edges = Vec::new();
    let mut issues = Vec::new();
    for (path, references) in sources {
        let (resolved, failures) = resolvers.dependencies(&path, references);
        edges.extend(resolved);
        issues.extend(failures);
    }
    (edges, issues)
}

fn diagnostic(rule: &Rule, issue: Issue) -> Result<Diagnostic> {
    let level = rule
        .level
        .ok_or_else(|| anyhow::anyhow!("architecture rule without a level"))?
        .name();
    let blocking = level == "error";
    Ok(Diagnostic {
        rule: rule.id.clone(),
        path: issue.path.to_string_lossy().into_owned(),
        level: level.into(),
        actual: None,
        limit: None,
        skill: if blocking {
            rule.error_skill.clone()
        } else {
            rule.warning_skill.clone()
        },
        message: issue.message,
        line: issue.line,
        symbol: None,
        rerun: String::new(),
    })
}
