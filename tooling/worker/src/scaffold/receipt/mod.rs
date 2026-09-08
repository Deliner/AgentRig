// DECISION: D023
#[cfg(test)]
mod tests;

pub use review_runner::artifacts::digest as checksum;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Ownership {
    Runtime,
    Asset,
    Configuration,
    Editable,
    Memory,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub sha256: String,
    pub ownership: Ownership,
    pub executable: bool,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub manifest_version: u32,
    pub package_version: String,
    pub config_schema: u32,
    pub files: BTreeMap<String, Entry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub local: BTreeMap<String, Option<String>>,
}
