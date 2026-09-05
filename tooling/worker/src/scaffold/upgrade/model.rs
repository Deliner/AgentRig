use super::super::package::manifest::Manifest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub resolved: String,
    pub sha256: Option<String>,
    pub mode: Option<u32>,
}
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
    pub files: BTreeMap<String, Change>,
    pub manifest: Manifest,
    pub checks: Vec<String>,
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
