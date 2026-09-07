mod analysis;
mod scope;

use super::{Contracts, Issue, source::References};
use crate::lint::{
    Diagnostic,
    config::{Rule, globs},
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
    // Selection exclusions limit findings, not the existence of dependency targets.
    let files: BTreeSet<_> = inventory::collect(&root, &globs(&[])?)?
        .files
        .into_iter()
        .collect();
    let settings = rule
        .architecture
        .as_ref()
        .expect("validated architecture settings");
    let rust_sources = analysis::rust_sources(&root, &files, settings);
    let (resolvers, mut issues) = analysis::Resolvers::new(&root, &files, &rust_sources, settings)?;
    let (contracts, contract_issues) = Contracts::load(&root, &scope.directories);
    issues.extend(contract_issues);
    issues.extend(contracts.check_inventory(&root, &scope.inventory));
    let sources = analysis::extract(&root, &scope.files);
    let (edges, source_issues) = dependencies(&resolvers, sources);
    issues.extend(source_issues);
    issues.extend(contracts.check(&edges));
    Ok(issues
        .into_iter()
        .map(|issue| diagnostic(rule, issue))
        .collect())
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

fn diagnostic(rule: &Rule, issue: Issue) -> Diagnostic {
    let level = rule.level.expect("validated architecture level").name();
    let blocking = level == "error";
    Diagnostic {
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
    }
}
