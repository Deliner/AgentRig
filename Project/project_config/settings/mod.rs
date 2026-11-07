mod capabilities;
mod validation;
pub use capabilities::Capabilities;
// DECISION: D005
pub use crate::shared::vcs::Settings as Git;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

// DECISION: D038
pub const FILE: &str = ".agentrig/project.yaml";
const MODULES: [&str; 6] = ["git", "commands", "checks", "hooks", "oracles", "tools"];
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
    #[serde(default)]
    pub tools: Tools,
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
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    pub argv: Vec<String>,
    #[serde(default = "dot")]
    pub cwd: String,
    #[serde(default)]
    pub accepts_args: bool,
    #[serde(default)]
    pub read_only: bool,
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
    pub deny: Vec<String>,
}
#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Lifetime {
    #[default]
    Task,
    Shared,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CheckKind {
    Command,
    Lint,
    Memory,
    Tools,
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

pub fn module(section: &str) -> String {
    format!(".agentrig/{section}.yaml")
}
pub fn read(root: &Path) -> Result<Config> {
    let mut sections: serde_json::Map<String, serde_json::Value> =
        crate::shared::yaml::read(&root.join(FILE))?;
    for section in MODULES {
        let path = module(section);
        ensure!(
            !sections.contains_key(section),
            "{FILE}: move {section} to {path}"
        );
        let present = root.join(&path).is_file();
        if present {
            sections.insert(
                section.into(),
                crate::shared::yaml::read(&root.join(&path))?,
            );
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
// DECISION: D045
pub fn sources(root: &Path) -> Vec<PathBuf> {
    let mut sources: Vec<PathBuf> = std::iter::once(FILE.to_owned())
        .chain(MODULES.map(module))
        .map(PathBuf::from)
        .collect();
    for (file, pointer) in [
        (FILE.to_owned(), "/paths/lint"),
        (module("hooks"), "/reminder"),
    ] {
        let document = crate::shared::yaml::read::<serde_json::Value>(&root.join(file)).ok();
        let referenced = document
            .as_ref()
            .and_then(|document| document.pointer(pointer))
            .and_then(serde_json::Value::as_str)
            .and_then(|path| relative(root, path).ok())
            .and_then(|path| path.strip_prefix(root).ok().map(Path::to_owned));
        sources.extend(referenced);
    }
    sources
}
pub struct ModuleFile {
    pub path: String,
    pub source: String,
}
pub fn files(config: &Config) -> Result<Vec<ModuleFile>> {
    let serde_json::Value::Object(mut sections) = serde_json::to_value(config)? else {
        anyhow::bail!("configuration must serialize to a mapping")
    };
    let mut files = Vec::new();
    for section in MODULES {
        let value = sections
            .remove(section)
            .ok_or_else(|| anyhow::anyhow!("missing configuration section {section}"))?;
        files.push(ModuleFile {
            path: module(section),
            source: crate::shared::yaml::encode(&value)?,
        });
    }
    files.push(ModuleFile {
        path: FILE.into(),
        source: crate::shared::yaml::encode(&sections)?,
    });
    Ok(files)
}
pub fn project_root(path: &Path) -> Result<PathBuf> {
    Ok(crate::shared::fs::canonicalize(path)?)
}
pub fn relative(root: &Path, value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    ensure!(
        !value.is_empty()
            && !path.is_absolute()
            && !path.components().any(|c| matches!(c, Component::ParentDir)),
        "path must stay relative to project: {value}"
    );
    let resolved = crate::shared::paths::resolve(&root.join(path))?;
    ensure!(resolved.starts_with(root), "path escapes project: {value}");
    Ok(resolved)
}
fn name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
