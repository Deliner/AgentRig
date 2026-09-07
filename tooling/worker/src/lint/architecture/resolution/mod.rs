//! Resolve references against the source inventory without executing project code.
pub mod javascript;
pub mod python;
pub mod rust;

use anyhow::{Result, bail, ensure};
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

fn normalize(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::Normal(name) => normalized.push(name),
            Component::ParentDir => ensure!(normalized.pop(), "module path escapes project root"),
            _ => bail!("module path must stay project-relative"),
        }
    }
    Ok(normalized)
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Resolved {
    pub files: BTreeSet<PathBuf>,
    pub external: Option<String>,
}
