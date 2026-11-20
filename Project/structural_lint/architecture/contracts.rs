use super::policy::{Declaration, Rooted, Zone};
use super::{Dependency, EnvironmentUse, Issue};
use crate::structural_lint::rule_identities::contract_settings::FILENAME;
use crate::structural_lint::targeting::file_discovery::Inventory;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

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
    pub fn check_inventory(&self, root: &Path, inventory: &Inventory) -> Vec<Issue> {
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
        inventory: &Inventory,
        edges: &[Dependency],
    ) -> Vec<Issue> {
        super::features::check(root, &self.declarations, inventory, edges)
    }

    // DECISION: D078
    pub fn check_zones(&self, environment: &BTreeMap<PathBuf, Vec<EnvironmentUse>>) -> Vec<Issue> {
        let mut issues = Vec::new();
        for (directory, declaration) in &self.declarations {
            let foreign_zone = declaration.zone() != Some(Zone::Pure);
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
            let budget = declaration.dependents();
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
