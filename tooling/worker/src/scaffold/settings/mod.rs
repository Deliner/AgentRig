mod capabilities;
mod validation;
pub use capabilities::Capabilities;
// DECISION: D005
pub use crate::vcs::Settings as Git;
use anyhow::{Context as _, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

pub const FILE: &str = "agentrig.yaml";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub frontend: Frontend,
    pub version: u32,
    pub runtime: String,
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
}
fn service() -> String {
    ".agentrig".into()
}
#[derive(Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Frontend {
    #[default]
    Codex,
    ClaudeCode,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub argv: Vec<String>,
    #[serde(default = "dot")]
    pub cwd: String,
    #[serde(default)]
    pub accepts_args: bool,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub lifetime: crate::jobs::Lifetime,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CheckKind {
    Command,
    Lint,
    Memory,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub id: String,
    pub kind: CheckKind,
    pub command: Option<String>,
    #[serde(default = "all")]
    pub include: Vec<String>,
    pub skill: String,
    #[serde(default)]
    pub warning: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected: Vec<AffectedTests>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AffectedTests {
    pub include: Vec<String>,
    pub targets: Vec<String>,
}
#[derive(Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Hooks {
    #[serde(default)]
    pub routes: Vec<Route>,
    pub reminder: Option<String>,
    pub discipline_skill: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub include: Vec<String>,
    pub skill: String,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Runner {
    Pytest,
    Cargo,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Oracle {
    pub check: String,
    pub runner: Runner,
    pub target: String,
}
fn dot() -> String {
    ".".into()
}
fn all() -> Vec<String> {
    vec!["**".into()]
}

pub fn read(root: &Path) -> Result<Config> {
    crate::yaml::read(&root.join(FILE)).with_context(|| format!("{FILE} required"))
}
pub fn relative(root: &Path, value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    ensure!(
        !value.is_empty()
            && !path.is_absolute()
            && !path.components().any(|c| matches!(c, Component::ParentDir)),
        "path must stay relative to project: {value}"
    );
    let resolved = crate::paths::resolve(&root.join(path))?;
    ensure!(resolved.starts_with(root), "path escapes project: {value}");
    Ok(resolved)
}
fn name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
