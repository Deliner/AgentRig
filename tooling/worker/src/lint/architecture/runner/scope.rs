use crate::lint::{
    config::{Rule, extension_matches},
    inventory::Inventory,
    selection,
};
use anyhow::{Result, ensure};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub struct Scope {
    pub directories: BTreeSet<PathBuf>,
    pub files: BTreeSet<PathBuf>,
}

impl Scope {
    pub fn new(rule: &Rule, inventory: &Inventory) -> Result<Self> {
        let mut directories: BTreeSet<_> = selection::select(rule, inventory)?
            .into_iter()
            .map(|selected| selected.path.to_path_buf())
            .collect();
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
        directories.retain(|directory| {
            files
                .iter()
                .any(|path| directory == Path::new(".") || path.starts_with(directory))
        });
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
        Ok(Self { directories, files })
    }
}
