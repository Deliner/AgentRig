// DECISION: D028
// DECISION: D043
use super::Issue;
use anyhow::{Result, ensure};
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
}

pub struct Contracts {
    declarations: BTreeMap<PathBuf, Declaration>,
}

impl Contracts {
    pub fn check_inventory(
        &self,
        root: &Path,
        inventory: &crate::structural_lint::inventory::Inventory,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();
        for (directory, declaration) in &self.declarations {
            issues.extend(super::inventory::check(
                root,
                directory,
                declaration,
                inventory,
            ));
        }
        issues
    }

    pub fn load(root: &Path, directories: &BTreeSet<PathBuf>) -> (Self, Vec<Issue>) {
        let mut contracts = Self {
            declarations: BTreeMap::new(),
        };
        let mut issues = Vec::new();
        for directory in directories {
            match load(root, directory) {
                Ok(declaration) => {
                    contracts
                        .declarations
                        .insert(directory.clone(), declaration);
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
}

fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

fn load(root: &Path, directory: &Path) -> Result<Declaration> {
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
    let declaration: Declaration = crate::shared::yaml::read(&path)?;
    ensure!(
        !declaration.purpose.trim().is_empty() && !declaration.purpose.contains(['\n', '\r']),
        "purpose must describe the directory responsibility on one nonempty line"
    );
    Ok(declaration)
}
