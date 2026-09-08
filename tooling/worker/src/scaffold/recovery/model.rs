use super::super::receipt::Manifest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use crate::scaffold::installation::State;
#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    Keep,
    Replace,
    Remove,
    Conflict,
}
#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Resolution {
    Keep,
    Replace,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub before: State,
    pub after: State,
    pub action: Action,
    pub reason: String,
    pub resolution: Option<Resolution>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub version: u32,
    pub project: String,
    pub from_version: String,
    pub to_version: String,
    pub baseline: String,
    #[serde(default = "legacy_service")]
    pub service: String,
    pub files: BTreeMap<String, Change>,
    pub manifest: Manifest,
    pub checks: Vec<String>,
}
fn legacy_service() -> String {
    ".worker".into()
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Journal {
    pub version: u32,
    pub plan: String,
    pub plan_sha256: String,
    pub phase: String,
    pub completed: Vec<String>,
    pub restored: Vec<String>,
    pub checks: BTreeMap<String, i32>,
    pub next_action: String,
}
