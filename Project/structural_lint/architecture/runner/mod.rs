mod analysis;
mod disallowed;
mod scope;

use super::{
    Contracts, Issue, Loaded, Neighbours, PYTHON_SEED, RUST_SEED, Settings, source::References,
};
use crate::shared::diagnostics::actionable_findings::Diagnostic;
use crate::shared::paths::glob_patterns::globs;
use crate::structural_lint::{
    rule_set::lint_config::Rule,
    targeting::file_discovery::{self, Inventory},
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

// DECISION: D143
pub fn edges(
    directory: super::Rooted<'_>,
    rule: &Rule,
    inventory: &Inventory,
) -> Result<Option<Neighbours>> {
    let unselected = !contains_directory(rule, directory.path, inventory)?;
    if unselected {
        return Ok(None);
    }
    let measured = measure(directory.root, rule, inventory)?;
    Ok(measured.map(|measured| super::graph::neighbours(&measured.analyzed.edges, directory.path)))
}

struct Measured<'r> {
    root: PathBuf,
    scope: Scope,
    settings: &'r Settings,
    issues: Vec<Issue>,
    analyzed: Analysis,
}

// DECISION: D143
fn measure<'r>(root: &Path, rule: &'r Rule, inventory: &Inventory) -> Result<Option<Measured<'r>>> {
    let scope = Scope::new(rule, inventory)?;
    let empty = scope.directories.is_empty();
    if empty {
        return Ok(None);
    }
    let root = crate::shared::fs_with_context::canonicalize(root)?;
    let no_exclusions = globs(&[])?;
    let files: BTreeSet<_> = file_discovery::collect(&root, &no_exclusions)?
        .files
        .into_iter()
        .collect();
    let settings = rule
        .architecture
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("architecture rule without settings"))?;
    let rust_sources = analysis::rust_sources(&root, &files);
    let (resolvers, issues) = analysis::Resolvers::new(&root, &files, &rust_sources, settings)?;
    let sources = analysis::extract(&root, &scope.files);
    let analyzed = dependencies(&resolvers, sources);
    Ok(Some(Measured {
        root,
        scope,
        settings,
        issues,
        analyzed,
    }))
}

pub fn run(root: &Path, rule: &Rule, inventory: &Inventory) -> Result<Vec<Diagnostic>> {
    let Some(Measured {
        root,
        scope,
        settings,
        mut issues,
        analyzed,
    }) = measure(root, rule, inventory)?
    else {
        return Ok(Vec::new());
    };
    let loaded: Loaded = Contracts::load(&root, &scope.directories);
    issues.extend(loaded.issues);
    let contracts = loaded.contracts;
    issues.extend(contracts.check_inventory(&root, &scope.inventory));
    issues.extend(analyzed.issues);
    issues.extend(unused_externals(settings, &scope, analyzed.used.as_ref()));
    issues.extend(super::graph::cycles(&analyzed.edges));
    // DECISION: D121
    issues.extend(super::graph::fan_out(&analyzed.edges, settings));
    let features = settings.feature_root.as_deref();
    issues.extend(contracts.check_features(features, &scope.inventory, &analyzed.edges));
    // DECISION: D078
    issues.extend(contracts.check_zones(&analyzed.environment));
    // DECISION: D079
    issues.extend(contracts.check_coupling(&analyzed.edges));
    issues
        .into_iter()
        .map(|issue| diagnostic(rule, issue))
        .collect()
}

struct Analysis {
    edges: Vec<super::Dependency>,
    issues: Vec<Issue>,
    used: Option<analysis::Externals>,
    environment: BTreeMap<PathBuf, Vec<super::EnvironmentUse>>,
}
fn dependencies(
    resolvers: &analysis::Resolvers<'_>,
    sources: BTreeMap<PathBuf, References>,
) -> Analysis {
    let mut edges = Vec::new();
    let mut issues = Vec::new();
    let mut used = analysis::Externals::default();
    let mut complete = true;
    let mut environment = BTreeMap::new();
    for (path, references) in sources {
        let analysis::Dependencies {
            edges: resolved,
            incomplete,
            violations,
            used: references_used,
            environment: references_environment,
        } = resolvers.dependencies(&path, references);
        edges.extend(resolved);
        complete &= incomplete.is_empty();
        issues.extend(incomplete);
        issues.extend(violations);
        used.rust.extend(references_used.rust);
        used.python.extend(references_used.python);
        // DECISION: D078
        let has_environment = !references_environment.is_empty();
        if has_environment {
            environment.insert(path, references_environment);
        }
    }
    Analysis {
        edges,
        issues,
        used: complete.then_some(used),
        environment,
    }
}

// DECISION: D076
// DECISION: D141
fn unused_externals(
    settings: &Settings,
    scope: &Scope,
    used: Option<&analysis::Externals>,
) -> Vec<Issue> {
    let Some(used) = used else {
        return Vec::new();
    };
    let owner = settings
        .feature_root
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    let searched = format!(
        "{count} files this rule reads under {root}",
        count = scope.files.len(),
        root = common_root(&scope.directories).display()
    );
    let rust = settings
        .external
        .rust
        .iter()
        .filter(|name| !RUST_SEED.contains(&name.as_str()) && !used.rust.contains(*name))
        .map(|name| format!("Rust dependency {name}"));
    let python = settings
        .external
        .python
        .iter()
        .filter(|name| !PYTHON_SEED.contains(&name.as_str()) && !used.python.contains(*name))
        .map(|name| format!("Python dependency {name}"));
    unused(&owner, &searched, rust.chain(python))
}

fn unused(owner: &Path, searched: &str, dependencies: impl Iterator<Item = String>) -> Vec<Issue> {
    dependencies
        .map(|dependency| Issue {
            path: owner.into(),
            line: None,
            message: format!(
                "declared external {dependency} is not named in the {searched}; \
                 add the file that uses it to this rule's selection, or remove the declaration"
            ),
        })
        .collect()
}

fn common_root(directories: &BTreeSet<PathBuf>) -> PathBuf {
    let mut paths = directories.iter();
    let first = paths.next().cloned().unwrap_or_default();
    let common = paths.fold(first, |common, path| {
        let shared = common
            .ancestors()
            .find(|ancestor| path.starts_with(ancestor));
        shared.map(Path::to_path_buf).unwrap_or_default()
    });
    let top = common.as_os_str().is_empty();
    if top { PathBuf::from(".") } else { common }
}

fn diagnostic(rule: &Rule, issue: Issue) -> Result<Diagnostic> {
    let level = rule
        .level
        .ok_or_else(|| anyhow::anyhow!("architecture rule without a level"))?;
    Ok(Diagnostic {
        rule: rule.id.clone(),
        path: issue.path.to_string_lossy().into_owned(),
        level,
        actual: None,
        limit: None,
        skill: rule.skill(level),
        message: issue.message,
        line: issue.line,
        symbol: None,
        rerun: String::new(),
    })
}
