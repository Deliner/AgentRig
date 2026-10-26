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

// DECISION: D038
pub const FILE: &str = ".agentrig/project.yaml";
const MODULES: [&str; 5] = ["git", "commands", "checks", "hooks", "oracles"];
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

fn module(section: &str) -> String {
    format!(".agentrig/{section}.yaml")
}
pub fn read(root: &Path) -> Result<Config> {
    let mut sections: serde_json::Map<String, serde_json::Value> =
        crate::yaml::read(&root.join(FILE)).with_context(|| format!("{FILE} required"))?;
    for section in MODULES {
        let path = module(section);
        ensure!(
            !sections.contains_key(section),
            "{FILE}: move {section} to {path}"
        );
        let present = root.join(&path).is_file();
        if present {
            sections.insert(section.into(), crate::yaml::read(&root.join(&path))?);
        }
    }
    serde_path_to_error::deserialize(serde_json::Value::Object(sections)).map_err(|error| {
        let field = error.path().to_string();
        let section = field.split(['.', '[']).next().unwrap_or_default();
        let owned = MODULES.contains(&section);
        let file = if owned { module(section) } else { FILE.into() };
        anyhow::anyhow!("{file}: {error}")
    })
}
pub fn files(config: &Config) -> Result<Vec<(String, String)>> {
    let serde_json::Value::Object(mut sections) = serde_json::to_value(config)? else {
        unreachable!("configuration serializes to a mapping")
    };
    let mut files = Vec::new();
    for section in MODULES {
        let value = sections
            .remove(section)
            .expect("serialized configuration section");
        files.push((module(section), crate::yaml::encode(&value)?));
    }
    files.push((FILE.into(), crate::yaml::encode(&sections)?));
    Ok(files)
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
