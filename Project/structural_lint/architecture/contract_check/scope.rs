use crate::shared::paths::glob_patterns::globs;
use crate::structural_lint::{
    rule_set::lint_config::{Rule, extension_matches},
    targeting::{file_discovery::Inventory, lint_targets},
};
use anyhow::{Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub struct Scope {
    pub directories: BTreeSet<PathBuf>,
    pub files: BTreeSet<PathBuf>,
    pub inventory: Inventory,
}

impl Scope {
    pub fn new(rule: &Rule, discovered: &Inventory) -> Result<Self> {
        let directories: BTreeSet<_> = lint_targets::select(rule, discovered)?
            .into_iter()
            .map(|selected| selected.path.to_path_buf())
            .collect();
        let inventory = selected_inventory(rule, discovered, &directories)?;
        let files: BTreeSet<_> = inventory
            .files
            .iter()
            .filter(|path| {
                let parent = path
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                    .unwrap_or(Path::new("."));
                directories.contains(parent) && extension_matches(path, &rule.extensions)
            })
            .cloned()
            .collect();
        let scope = Self {
            directories,
            files,
            inventory,
        };
        scope.validate_roots(rule, discovered)?;
        Ok(scope)
    }

    // DECISION: D189
    fn selects(&self, rule: &Rule, inventory: &Inventory, directory: &Path) -> Result<bool> {
        let present = inventory.directories.contains_key(directory);
        if present {
            return Ok(self.directories.contains(directory));
        }
        let included = globs(&rule.include)?.is_match(directory);
        Ok(included && !globs(&rule.exclude)?.is_match(directory))
    }

    fn validate_roots(&self, rule: &Rule, inventory: &Inventory) -> Result<()> {
        let needs_rust_root = self
            .files
            .iter()
            .any(|path| path.extension().is_some_and(|ext| ext == "rs"));
        let settings = rule
            .architecture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("architecture rule without settings"))?;
        ensure!(
            !needs_rust_root || !settings.rust_roots.is_empty(),
            "{}: selected Rust sources need architecture.rust_roots",
            rule.id
        );
        let selected_root = match &settings.feature_root {
            Some(root) => self.selects(rule, inventory, root)?,
            None => true,
        };
        let unselected_root = !selected_root;
        ensure!(
            !unselected_root,
            "{}: architecture.feature_root must be a selected directory",
            rule.id
        );
        Ok(())
    }
}

fn selected_inventory(
    rule: &Rule,
    inventory: &Inventory,
    selected: &BTreeSet<PathBuf>,
) -> Result<Inventory> {
    let excluded = globs(&rule.exclude)?;
    let mut directories = BTreeMap::new();
    for directory in selected {
        let entries: BTreeSet<String> = inventory.directories[directory]
            .iter()
            .filter(|name| {
                let path = directory.join(name);
                let path = path.trim_prefix(".");
                let child = inventory.directories.contains_key(path);
                let included = !excluded.is_match(path);
                let reachable = !child || selected.contains(path);
                included && reachable
            })
            .cloned()
            .collect();
        directories.insert(directory.clone(), entries);
    }
    let files: Vec<PathBuf> = inventory
        .files
        .iter()
        .filter(|path| !excluded.is_match(path))
        .cloned()
        .collect();
    Ok(Inventory { files, directories })
}
