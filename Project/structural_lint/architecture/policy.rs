// DECISION: D028
// DECISION: D043
// DECISION: D044
use super::{Dependency, EnvironmentUse, Issue};
use crate::shared::fs;
use anyhow::{Result, ensure};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

pub const FILENAME: &str = "architecture.yaml";

// DECISION: D070
const PLACEHOLDER_PURPOSES: &[&str] = &[
    "todo",
    "tbd",
    "fixme",
    "wip",
    "xxx",
    "na",
    "none",
    "placeholder",
    "description",
    "purpose",
    "module",
    "directory",
    "package",
    "folder",
    "file",
    "code",
    "stuff",
    "misc",
    "various",
    "temporary",
];

// DECISION: D078
#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Zone {
    Pure,
}

// DECISION: D079
const DEFAULT_DEPENDENTS: usize = 6;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Declaration {
    purpose: String,
    #[serde(default)]
    files: BTreeMap<String, String>,
    #[serde(default)]
    directories: BTreeMap<String, String>,
    #[serde(default, rename = "type")]
    kind: Option<super::features::Kind>,
    // DECISION: D078
    #[serde(default)]
    zone: Option<Zone>,
    // DECISION: D079
    #[serde(default)]
    dependents: Option<usize>,
}
// DECISION: D129
impl Declaration {
    pub(super) fn files(&self) -> &BTreeMap<String, String> {
        &self.files
    }
    pub(super) fn directories(&self) -> &BTreeMap<String, String> {
        &self.directories
    }
    pub(super) fn kind(&self) -> Option<super::features::Kind> {
        self.kind
    }
}

pub struct Contracts {
    declarations: BTreeMap<PathBuf, Declaration>,
}

// DECISION: D083
// DECISION: D104
fn zone_message(effect: &str, path: &str) -> String {
    format!("zone `pure` directory must not {effect}; move the effect out or drop the zone: {path}")
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

    pub fn check_features(
        &self,
        root: Option<&Path>,
        inventory: &crate::structural_lint::inventory::Inventory,
        edges: &[Dependency],
    ) -> Vec<Issue> {
        super::features::check(root, &self.declarations, inventory, edges)
    }

    // DECISION: D078
    pub fn check_zones(&self, environment: &BTreeMap<PathBuf, Vec<EnvironmentUse>>) -> Vec<Issue> {
        let mut issues = Vec::new();
        for (directory, declaration) in &self.declarations {
            let foreign_zone = declaration.zone != Some(Zone::Pure);
            if foreign_zone {
                continue;
            }
            for (file, uses) in environment {
                let parent = file
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                    .unwrap_or_else(|| Path::new("."));
                let other_directory = parent != directory.as_path();
                if other_directory {
                    continue;
                }
                issues.extend(uses.iter().filter_map(|use_site| {
                    let effect = super::effect(&use_site.path)?;
                    Some(Issue {
                        path: file.clone(),
                        line: Some(use_site.line),
                        message: zone_message(effect, &use_site.path),
                    })
                }));
            }
        }
        issues
    }

    // DECISION: D079
    pub fn check_coupling(&self, edges: &[Dependency]) -> Vec<Issue> {
        let mut dependents: BTreeMap<&Path, BTreeSet<&Path>> = BTreeMap::new();
        for edge in edges {
            if edge.module_declaration {
                continue;
            }
            let (Some(source), Some(target)) = (edge.source.parent(), edge.target.parent()) else {
                continue;
            };
            let same_directory = source == target;
            if same_directory {
                continue;
            }
            dependents.entry(target).or_default().insert(source);
        }
        let mut issues = Vec::new();
        for (directory, declaration) in &self.declarations {
            let budget = declaration.dependents.unwrap_or(DEFAULT_DEPENDENTS);
            let incoming = dependents.get(directory.as_path()).map_or(0, BTreeSet::len);
            let over_budget = incoming > budget;
            if over_budget {
                issues.push(Issue {
                    path: directory.join(FILENAME),
                    line: None,
                    message: format!(
                        "directory is depended on by {incoming} other directories (budget {budget}); reduce incoming coupling or raise the declared dependents budget"
                    ),
                });
            }
        }
        issues
    }

    pub fn load(root: &Path, directories: &BTreeSet<PathBuf>) -> Loaded {
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
        Loaded { contracts, issues }
    }
}

pub struct Loaded {
    pub contracts: Contracts,
    pub issues: Vec<Issue>,
}

pub struct Description {
    pub purpose: String,
    pub role: String,
}
pub fn describe(root: &Path, file: &Path) -> Option<Description> {
    let directory = file
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    let declaration = load(root, directory.unwrap_or(Path::new("."))).ok()?;
    let role = declaration.files.get(file.file_name()?.to_str()?)?.clone();
    Some(Description {
        purpose: declaration.purpose,
        role,
    })
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
        fs::canonicalize(&path)?.starts_with(fs::canonicalize(root)?),
        "contract escapes project root"
    );
    let declaration: Declaration = crate::shared::yaml::read(&path)?;
    let purpose = declaration.purpose.trim();
    ensure!(
        !purpose.is_empty() && !declaration.purpose.contains(['\n', '\r']),
        "purpose must describe the directory responsibility on one nonempty line"
    );
    let gist = purpose
        .trim_matches(|character: char| !character.is_alphanumeric())
        .to_ascii_lowercase();
    ensure!(
        !gist.is_empty() && !PLACEHOLDER_PURPOSES.contains(&gist.as_str()),
        "purpose must state the real directory responsibility, not a placeholder"
    );
    Ok(declaration)
}
