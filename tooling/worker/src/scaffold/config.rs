mod capabilities;
mod validation;
pub use capabilities::{Capabilities, Resource};
// DECISION: D005
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
    pub version: u32,
    pub runtime: String,
    pub config_skill: String,
    pub paths: Paths,
    #[serde(default)]
    pub processes: Processes,
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
    pub sources: Vec<String>,
    pub memory: String,
    pub skills: String,
    pub lint: String,
    pub runtime: String,
}
#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Processes {
    #[serde(default)]
    pub foreground: Containment,
}
#[derive(Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Containment {
    #[default]
    ProcessGroup,
    Systemd,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Git {
    pub base: String,
    pub prefix: String,
}
impl Default for Git {
    fn default() -> Self {
        Self {
            base: "main".into(),
            prefix: "feature/".into(),
        }
    }
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
    pub lifetime: agentrig::jobs::Lifetime,
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

pub struct Context {
    pub root: PathBuf,
    pub config: Config,
}
impl Context {
    pub fn load(root: &Path) -> Result<Self> {
        Self::load_for(root, false)
    }
    pub fn load_for(root: &Path, recovery: bool) -> Result<Self> {
        let root = root.canonicalize()?;
        let mut config = read(&root)?;
        let pin = config.runtime.clone();
        let supported_pin = pin == VERSION || pin == super::upgrade::release::FROM;
        let recovering = recovery && supported_pin && super::upgrade::recovery::active(&root)?;
        if recovering {
            config.runtime = VERSION.into();
        }
        let validation = if recovering {
            config.validate_structure(&root)
        } else {
            config.validate(&root)
        };
        validation.with_context(|| {
            format!(
                "configuration invalid. ACTION: Apply {}",
                config.config_skill
            )
        })?;
        config.runtime = pin;
        Ok(Self { root, config })
    }
    pub fn path(&self, value: &str) -> Result<PathBuf> {
        relative(&self.root, value)
    }
}
pub fn read(root: &Path) -> Result<Config> {
    review_runner::config::yaml::read(&root.join(FILE)).with_context(|| {
        format!("{FILE} required; use explicit upgrade for legacy worker.toml, no format fallback")
    })
}
// Resolve existing ancestors too: symlinks must not escape the selected project tree.
pub fn relative(root: &Path, value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    ensure!(
        !value.is_empty()
            && !path.is_absolute()
            && !path.components().any(|c| matches!(c, Component::ParentDir)),
        "path must stay relative to project: {value}"
    );
    let resolved = crate::util::resolve(&root.join(path))?;
    ensure!(resolved.starts_with(root), "path escapes project: {value}");
    Ok(resolved)
}
fn name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn branch_name(value: &str) -> bool {
    !value.is_empty()
        && value != "HEAD"
        && !value.starts_with('-')
        && !value.ends_with('.')
        && !value.contains("..")
        && !value.contains("@{")
        && !value
            .bytes()
            .any(|byte| byte <= 32 || byte == 127 || b"~^:?*[\\".contains(&byte))
        && value
            .split('/')
            .all(|part| !part.is_empty() && !part.starts_with('.') && !part.ends_with(".lock"))
}
