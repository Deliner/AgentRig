use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

const LIBRARY_KINDS: [&str; 6] = ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"];

#[derive(Deserialize)]
struct Metadata {
    workspace_root: PathBuf,
    workspace_default_members: Vec<String>,
    packages: Vec<Package>,
}
#[derive(Deserialize)]
struct Package {
    id: String,
    targets: Vec<Target>,
}
#[derive(Deserialize)]
struct Target {
    kind: Vec<String>,
}

pub struct Workspace {
    pub root: PathBuf,
    pub library: bool,
}

pub fn read(metadata: &[u8]) -> Result<Workspace> {
    let metadata: Metadata = serde_json::from_slice(metadata).context("parse cargo metadata")?;
    let selected = metadata.packages.iter().filter(|package| {
        let members = &metadata.workspace_default_members;
        members.contains(&package.id)
    });
    let mut kinds = selected
        .flat_map(|package| &package.targets)
        .flat_map(|target| &target.kind);
    let library = kinds.any(|kind| LIBRARY_KINDS.contains(&kind.as_str()));
    Ok(Workspace {
        root: metadata.workspace_root,
        library,
    })
}
