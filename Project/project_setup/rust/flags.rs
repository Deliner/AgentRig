use crate::shared::fs_with_context as fs;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct CargoConfig {
    #[serde(default)]
    build: Build,
}
#[derive(Default, Deserialize)]
struct Build {
    rustflags: Option<Flags>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Flags {
    Line(String),
    List(Vec<String>),
}

// DECISION: D182
pub fn build_rustflags(root: &Path) -> Result<Vec<String>> {
    let path = root.join(".cargo/config.toml");
    let Some(source) = fs::present(fs::read_to_string(&path))? else {
        return Ok(Vec::new());
    };
    let config: CargoConfig =
        toml::from_str(&source).with_context(|| format!("parse {}", path.display()))?;
    let flags = match config.build.rustflags {
        None => Vec::new(),
        Some(Flags::Line(line)) => line.split_whitespace().map(str::to_owned).collect(),
        Some(Flags::List(list)) => list,
    };
    Ok(flags)
}
