use crate::shared::fs_with_context as fs;
use crate::structural_lint::CargoLayout;
use anyhow::{Context, Result, ensure};
use std::path::{Path, PathBuf};

// DECISION: D154
const SETTINGS: &str = include_str!("../../clippy.toml");
const FILES: [&str; 2] = [".clippy.toml", "clippy.toml"];

pub struct Settings {
    path: PathBuf,
    contents: String,
}

impl Settings {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn write(&self) -> Result<()> {
        Ok(fs::write(&self.path, &self.contents)?)
    }
}

// DECISION: D154
pub fn settings(root: &Path, rust: &CargoLayout) -> Result<Settings> {
    let manifest = root.join(&rust.manifest);
    let directory = manifest
        .parent()
        .context("Cargo manifest has no directory")?;
    let ancestors = directory
        .ancestors()
        .take_while(|ancestor| ancestor.starts_with(root));
    let (path, existing) = match effective(ancestors) {
        Some(path) => {
            let existing = fs::read_to_string(&path)?;
            (path, existing)
        }
        None => (root.join("clippy.toml"), String::new()),
    };
    let present: toml::Table =
        toml::from_str(&existing).with_context(|| format!("parse {}", path.display()))?;
    let required: toml::Table = toml::from_str(SETTINGS)?;
    let mut missing = String::new();
    for (key, value) in required {
        match present.get(&key) {
            Some(found) => ensure!(
                *found == value,
                "init collision: {} sets {key} = {found}, AgentRig requires {value}; existing files were preserved",
                path.display()
            ),
            None => missing.push_str(&format!("{key} = {value}\n")),
        }
    }
    Ok(Settings {
        contents: missing + &existing,
        path,
    })
}

fn effective<'a>(directories: impl Iterator<Item = &'a Path>) -> Option<PathBuf> {
    directories
        .flat_map(|ancestor| FILES.map(|name| ancestor.join(name)))
        .find(|path| path.is_file())
}
