use crate::shared::diagnostics::Level;
use crate::structural_lint::rule_identities::{Junctions, Kind, Pair, Settings};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// DECISION: D016
// DECISION: D017
// DECISION: D018
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub config_skill: String,
    pub skill_root: Option<PathBuf>,
    #[serde(default)]
    pub exclude: Vec<String>,
    pub rules: Vec<Rule>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
    pub id: String,
    pub kind: Kind,
    pub target: String,
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    pub warning: Option<usize>,
    pub error: Option<usize>,
    pub level: Option<Level>,
    pub warning_skill: String,
    pub error_skill: String,
    #[serde(default)]
    pub overrides: Vec<Override>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<Settings>,
    // DECISION: D156
    #[serde(default, skip_serializing_if = "Junctions::mixed")]
    pub junctions: Junctions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pairs: Option<Vec<Pair>>,
    // DECISION: D198
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_languages: Option<Vec<String>>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Override {
    pub include: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    pub warning: Option<usize>,
    pub error: Option<usize>,
}
fn enabled_by_default() -> bool {
    true
}
