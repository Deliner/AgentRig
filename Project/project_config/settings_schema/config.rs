use super::access::Access;
use super::arguments::{self, Arguments};
use super::capabilities::Capabilities;
use super::checks::Check;
use super::flag::Flag;
use super::key::Key;
use crate::shared::vcs::Settings as Git;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub runtime: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<Key>,
    pub config_skill: String,
    pub paths: Paths,
    #[serde(default)]
    pub capabilities: Capabilities,
    #[serde(default)]
    pub git: Git,
    #[serde(default)]
    pub commands: BTreeMap<String, Command>,
    #[serde(default)]
    pub checks: Vec<Check>,
    #[serde(default)]
    pub hooks: Hooks,
    #[serde(default)]
    pub oracles: BTreeMap<String, Oracle>,
    #[serde(default)]
    pub tools: Tools,
    #[serde(default)]
    pub rust: Rust,
}
// DECISION: D182
// DECISION: D239
#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rust {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<String>,
    pub rustflags: Vec<String>,
}
#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tools {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recognize: Vec<String>,
    #[serde(default)]
    pub locations: Vec<String>,
    #[serde(default)]
    pub entries: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub removed: BTreeMap<String, Removed>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Removed {
    pub purpose: String,
    pub reason: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Paths {
    #[serde(default = "service")]
    pub service: String,
    pub sources: Vec<String>,
    pub memory: String,
    pub skills: String,
    pub lint: String,
    pub runtime: String,
}
impl Paths {
    pub fn service_path(&self, path: &str) -> String {
        format!("{}/{path}", self.service)
    }
    // DECISION: D218
    pub fn source_matcher(&self) -> Result<globset::GlobSet> {
        crate::shared::paths::globs(&self.sources)
    }
}
impl Config {
    // DECISION: D233
    pub fn keep_matcher(&self) -> Result<globset::GlobSet> {
        crate::shared::paths::globs(&self.git.keep)
    }
}
fn service() -> String {
    ".agentrig".into()
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    pub argv: Vec<String>,
    #[serde(default = "dot")]
    pub cwd: String,
    #[serde(
        default,
        deserialize_with = "arguments::deserialize",
        serialize_with = "arguments::serialize"
    )]
    pub accepts_args: Arguments,
    #[serde(default)]
    pub access: Access,
    #[serde(default)]
    pub lifetime: Lifetime,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub programs: BTreeMap<String, Program>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Views>,
}
#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Views {
    #[serde(default)]
    pub success: View,
    #[serde(default)]
    pub failure: View,
}
#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct View {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matching: Vec<String>,
    #[serde(default)]
    pub tail: usize,
}
#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subcommands: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<Flag>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access: Option<Access>,
}
#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Lifetime {
    #[default]
    Task,
    Shared,
}
#[derive(Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Hooks {
    #[serde(default)]
    pub routes: Vec<Route>,
    pub reminder: Option<String>,
    pub discipline_skill: Option<String>,
    // DECISION: D213
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protected: Vec<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub include: Vec<String>,
    pub skill: String,
    #[serde(default = "changes")]
    pub events: Vec<RouteEvent>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub block: bool,
}
// DECISION: D047
#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RouteEvent {
    Edit,
    Shell,
    Check,
}
fn changes() -> Vec<RouteEvent> {
    vec![RouteEvent::Edit, RouteEvent::Shell]
}
#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Runner {
    Pytest,
    Cargo,
}
#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Oracle {
    pub check: String,
    pub runner: Runner,
    pub target: String,
}
fn dot() -> String {
    ".".into()
}
