// DECISION: D028
// DECISION: D043
// DECISION: D044
use super::{Dependency, EnvironmentUse, Issue};
use crate::shared::fs_with_context as fs;
use crate::structural_lint::rule_identities::contract_settings::FILENAME;
use anyhow::{Result, ensure};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

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
fn zone_message(path: &str) -> Option<String> {
    let effect = super::effect(path)?;
    Some(format!(
        "zone `pure` directory must not {effect}; move the effect out or drop the zone: {path}"
    ))
}

impl Contracts {
    pub fn check_inventory(
        &self,
        root: &Path,
        inventory: &crate::structural_lint::targeting::file_discovery::Inventory,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();
        for (path, declaration) in &self.declarations {
            let rooted = Rooted { root, path };
            issues.extend(super::inventory::check(rooted, declaration, inventory));
        }
        issues
    }

    pub fn check_features(
        &self,
        root: Option<&Path>,
        inventory: &crate::structural_lint::targeting::file_discovery::Inventory,
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
            'files: for (file, uses) in environment {
                let parent = file
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                    .unwrap_or_else(|| Path::new("."));
                let other_directory = parent != directory.as_path();
                if other_directory {
                    continue 'files;
                }
                issues.extend(uses.iter().filter_map(|use_site| {
                    let message = zone_message(&use_site.path)?;
                    Some(Issue {
                        path: file.clone(),
                        line: Some(use_site.line),
                        message,
                    })
                }));
            }
        }
        issues
    }

    // DECISION: D079
    // DECISION: D142
    pub fn check_coupling(&self, edges: &[Dependency]) -> Vec<Issue> {
        let dependents = super::graph::dependents(edges);
        let mut issues = Vec::new();
        for (directory, declaration) in &self.declarations {
            let budget = declaration.dependents.unwrap_or(DEFAULT_DEPENDENTS);
            let Some(sources) = dependents.get(directory.as_path()) else {
                continue;
            };
            let over_budget = sources.len() > budget;
            if over_budget {
                let named: Vec<_> = sources
                    .iter()
                    .map(|(source, edge)| {
                        let source = super::graph::directory_key(source);
                        format!(
                            "{source} ({evidence})",
                            source = source.display(),
                            evidence = super::graph::evidence(edge)
                        )
                    })
                    .collect();
                issues.push(Issue {
                    path: directory.join(FILENAME),
                    line: None,
                    message: format!(
                        "directory is depended on by {count} other directories (budget {budget}): {listed}; reduce incoming coupling or raise the declared dependents budget",
                        count = sources.len(),
                        listed = named.join(", ")
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
        for path in directories {
            let directory = path;
            match (Rooted { root, path }).declaration() {
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

#[derive(Clone, Copy)]
pub struct Rooted<'a> {
    pub root: &'a Path,
    pub path: &'a Path,
}

impl Rooted<'_> {
    pub(super) fn absolute(self) -> PathBuf {
        self.root.join(self.path)
    }

    pub fn describe(self) -> Option<Description> {
        let root = self.root;
        let path = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let declaration = Rooted { root, path }.declaration().ok()?;
        let role = declaration
            .files
            .get(self.path.file_name()?.to_str()?)?
            .clone();
        Some(Description {
            purpose: declaration.purpose,
            role,
        })
    }

    fn declaration(self) -> Result<Declaration> {
        let Self {
            root,
            path: directory,
        } = self;
        ensure!(
            directory == Path::new(".") || relative(directory),
            "invalid directory path"
        );
        let path = self.absolute().join(FILENAME);
        ensure!(
            fs::symlink_metadata(&path)?.is_file(),
            "contract must be a regular file"
        );
        ensure!(
            fs::canonicalize(&path)?.starts_with(fs::canonicalize(root)?),
            "contract escapes project root"
        );
        let declaration: Declaration = crate::shared::strict_yaml::read(&path)?;
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
}

fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            // DECISION: D156
            .all(|part| matches!(part, Component::Normal(_)))
}
