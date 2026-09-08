// DECISION: D028
use super::{Dependency, Issue};
use crate::lint::config::globs;
use anyhow::{Result, ensure};
use globset::GlobSet;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

pub const FILENAME: &str = "architecture.yaml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Declaration {
    purpose: String,
    #[serde(default)]
    pub files: BTreeMap<String, String>,
    #[serde(default)]
    pub directories: BTreeMap<String, String>,
    #[serde(default)]
    allow: Vec<String>,
    #[serde(default)]
    deny: Vec<String>,
    #[serde(default)]
    public: Vec<String>,
}

struct Policy {
    declaration: Declaration,
    allow: GlobSet,
    deny: GlobSet,
    public: GlobSet,
}

pub struct Contracts {
    policies: BTreeMap<PathBuf, Policy>,
}

impl Contracts {
    pub fn check_inventory(
        &self,
        root: &Path,
        inventory: &crate::lint::inventory::Inventory,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();
        for (directory, policy) in &self.policies {
            issues.extend(super::inventory::check(
                root,
                directory,
                &policy.declaration,
                inventory,
            ));
        }
        issues
    }

    /// Directory paths are project-relative; the root is represented by `.`.
    pub fn load(root: &Path, directories: &BTreeSet<PathBuf>) -> (Self, Vec<Issue>) {
        let mut contracts = Self {
            policies: BTreeMap::new(),
        };
        let mut issues = Vec::new();
        for directory in directories {
            match load(root, directory) {
                Ok(policy) => {
                    contracts.policies.insert(directory.clone(), policy);
                }
                Err(error) => issues.push(Issue {
                    path: directory.join(FILENAME),
                    line: None,
                    message: format!("invalid or missing directory contract: {error:#}"),
                }),
            }
        }
        (contracts, issues)
    }

    pub(super) fn check_edge(&self, edge: &Dependency, issues: &mut Vec<Issue>) {
        for (directory, policy) in &self.policies {
            check_boundary(directory, policy, edge, issues);
        }
    }
}

fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

fn load(root: &Path, directory: &Path) -> Result<Policy> {
    ensure!(
        directory == Path::new(".") || relative(directory),
        "invalid directory path"
    );
    let path = root.join(directory).join(FILENAME);
    ensure!(
        fs::symlink_metadata(&path)?.is_file(),
        "contract must be a regular file"
    );
    ensure!(
        path.canonicalize()?.starts_with(root.canonicalize()?),
        "contract escapes project root"
    );
    let declaration: Declaration = review_runner::config::yaml::read(&path)?;
    ensure!(
        !declaration.purpose.trim().is_empty() && !declaration.purpose.contains(['\n', '\r']),
        "purpose must describe the directory responsibility on one nonempty line"
    );
    Ok(Policy {
        allow: globs(&declaration.allow)?,
        deny: globs(&declaration.deny)?,
        public: globs(&declaration.public)?,
        declaration,
    })
}

fn inside(path: &Path, directory: &Path) -> bool {
    directory == Path::new(".") || path.starts_with(directory)
}

fn check_boundary(directory: &Path, policy: &Policy, edge: &Dependency, issues: &mut Vec<Issue>) {
    let source_inside = inside(&edge.source, directory);
    let target_inside = inside(&edge.target, directory);
    let outbound = source_inside && !target_inside;
    let inbound = !source_inside && target_inside;
    let forbidden =
        outbound && (!policy.allow.is_match(&edge.target) || policy.deny.is_match(&edge.target));
    if forbidden {
        issues.push(Issue::dependency(
            edge,
            format!(
                "forbidden dependency {} -> {} across {} (allow/deny)",
                edge.source.display(),
                edge.target.display(),
                directory.display()
            ),
        ));
    }
    if inbound {
        let relative = edge.target.strip_prefix(directory).unwrap();
        let private = !policy.public.is_match(relative);
        if private {
            issues.push(Issue::dependency(
                edge,
                format!(
                    "private access {} -> {} across {} (public)",
                    edge.source.display(),
                    edge.target.display(),
                    directory.display()
                ),
            ));
        }
    }
}
