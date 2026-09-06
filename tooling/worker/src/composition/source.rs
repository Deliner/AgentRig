use anyhow::{Result, ensure};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Root {
    #[serde(default)]
    pub packages: Vec<Import>,
    #[serde(default)]
    pub overrides: Vec<String>,
    #[serde(flatten)]
    pub configuration: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub schema_version: u32,
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub packages: Vec<Import>,
    #[serde(default)]
    pub overrides: Vec<String>,
    pub configuration: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Import {
    pub path: PathBuf,
    pub id: Option<String>,
    pub version: Option<String>,
}

impl Package {
    pub fn validate(&self, reference: &Import) -> Result<()> {
        ensure!(
            self.schema_version == 1,
            "unsupported package schema_version"
        );
        ensure!(
            !self.id.is_empty()
                && self
                    .id
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric()
                        || matches!(character, '-' | '_')),
            "package id must use letters, digits, hyphens or underscores"
        );
        ensure!(!self.version.trim().is_empty(), "package version required");
        ensure!(
            reference.id.as_ref().is_none_or(|id| id == &self.id),
            "package ID mismatch: expected {:?}, found {}",
            reference.id,
            self.id
        );
        ensure!(
            reference
                .version
                .as_ref()
                .is_none_or(|version| version == &self.version),
            "package version mismatch for {}: expected {:?}, found {}",
            self.id,
            reference.version,
            self.version
        );
        Ok(())
    }
}
