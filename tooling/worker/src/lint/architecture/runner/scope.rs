use crate::lint::{
    config::{Rule, extension_matches, globs},
    inventory::Inventory,
    selection,
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
    pub fn new(rule: &Rule, inventory: &Inventory) -> Result<Self> {
        let directories: BTreeSet<_> = selection::select(rule, inventory)?
            .into_iter()
            .map(|selected| selected.path.to_path_buf())
            .collect();
        let inventory = selected_inventory(rule, inventory, &directories)?;
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
        let needs_rust_root = files
            .iter()
            .any(|path| path.extension().is_some_and(|ext| ext == "rs"));
        let settings = rule
            .architecture
            .as_ref()
            .expect("validated architecture settings");
        ensure!(
            !needs_rust_root || !settings.rust_roots.is_empty(),
            "{}: selected Rust sources need architecture.rust_roots",
            rule.id
        );
        Ok(Self {
            directories,
            files,
            inventory,
        })
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
        let entries = inventory.directories[directory]
            .iter()
            .filter(|name| {
                let path = directory.join(name);
                let path = path.strip_prefix(".").unwrap_or(&path);
                let child = inventory.directories.contains_key(path);
                !excluded.is_match(path) && (!child || selected.contains(path))
            })
            .cloned()
            .collect();
        directories.insert(directory.clone(), entries);
    }
    let files = inventory
        .files
        .iter()
        .filter(|path| !excluded.is_match(path))
        .cloned()
        .collect();
    Ok(Inventory { files, directories })
}
