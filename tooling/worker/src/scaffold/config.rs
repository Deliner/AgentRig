use crate::lint::config::{globs, skill};
use anyhow::{Context as _, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Component, Path, PathBuf},
};

pub const FILE: &str = "worker.toml";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub runtime: String,
    pub config_skill: String,
    pub paths: Paths,
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
        let root = root.canonicalize()?;
        let path = root.join(FILE);
        let source =
            fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let config: Config = toml::from_str(&source).context("worker.toml schema")?;
        config.validate(&root).with_context(|| {
            format!(
                "configuration invalid. ACTION: Apply {}",
                config.config_skill
            )
        })?;
        Ok(Self { root, config })
    }
    pub fn path(&self, value: &str) -> Result<PathBuf> {
        relative(&self.root, value)
    }
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
impl Config {
    fn validate(&self, root: &Path) -> Result<()> {
        ensure!(self.version == 1, "version: supported schema is 1");
        ensure!(
            self.runtime == VERSION,
            "runtime: project pins {}, binary is {VERSION}; install the pinned binary",
            self.runtime
        );
        skill(root, &self.config_skill).context("config_skill")?;
        for (label, value) in [
            ("memory", &self.paths.memory),
            ("skills", &self.paths.skills),
            ("lint", &self.paths.lint),
            ("runtime", &self.paths.runtime),
        ] {
            relative(root, value).with_context(|| format!("paths.{label}"))?;
        }
        ensure!(
            !self.paths.sources.is_empty(),
            "paths.sources cannot be empty"
        );
        globs(&self.paths.sources).context("paths.sources")?;
        ensure!(
            !self.git.base.trim().is_empty() && !self.git.base.starts_with('-'),
            "git.base must name a branch"
        );
        ensure!(
            !self.git.prefix.is_empty() && !self.git.prefix.starts_with('-'),
            "git.prefix must be nonempty"
        );
        for (id, command) in &self.commands {
            ensure!(name(id), "commands.{id}: invalid command name");
            ensure!(
                !command.argv.is_empty() || command.accepts_args,
                "commands.{id}: empty argv requires accepts_args"
            );
            ensure!(
                command.argv.first().is_none_or(|s| !s.is_empty()),
                "commands.{id}: empty executable"
            );
            relative(root, &command.cwd).with_context(|| format!("commands.{id}.cwd"))?;
        }
        let mut ids = HashSet::new();
        for check in &self.checks {
            ensure!(
                name(&check.id) && ids.insert(check.id.as_str()),
                "checks: invalid or duplicate ID {}",
                check.id
            );
            globs(&check.include).with_context(|| format!("checks.{}.include", check.id))?;
            ensure!(
                !check.include.is_empty(),
                "checks.{}: empty include",
                check.id
            );
            skill(root, &check.skill).with_context(|| format!("checks.{}.skill", check.id))?;
            match (&check.kind, &check.command) {
                (CheckKind::Command, Some(command)) => {
                    let spec = self.commands.get(command).with_context(|| {
                        format!("checks.{}: unknown command {command}", check.id)
                    })?;
                    ensure!(
                        !spec.argv.is_empty(),
                        "checks.{}: command needs an executable",
                        check.id
                    );
                }
                (CheckKind::Command, None) => {
                    anyhow::bail!("checks.{}: command is required", check.id)
                }
                (_, Some(_)) => {
                    anyhow::bail!("checks.{}: builtin check cannot specify command", check.id)
                }
                _ => {}
            }
        }
        for route in &self.hooks.routes {
            ensure!(!route.include.is_empty(), "hooks.routes: include required");
            globs(&route.include).context("hooks.routes.include")?;
            skill(root, &route.skill).context("hooks.routes.skill")?;
        }
        if let Some(path) = &self.hooks.discipline_skill {
            skill(root, path).context("hooks.discipline_skill")?;
        }
        if let Some(path) = &self.hooks.reminder {
            ensure!(
                self.hooks.discipline_skill.is_some(),
                "hooks.reminder requires hooks.discipline_skill"
            );
            let source = fs::read_to_string(relative(root, path)?).context("hooks.reminder")?;
            let value: serde_json::Value =
                serde_json::from_str(&source).context("hooks.reminder JSON")?;
            let attention = value["attention_interval_tokens"]
                .as_u64()
                .context("hooks.reminder.attention_interval_tokens")?;
            let full = value["full_refresh_interval_tokens"]
                .as_u64()
                .context("hooks.reminder.full_refresh_interval_tokens")?;
            ensure!(
                attention > 0 && full > attention,
                "hooks.reminder: require 0 < attention < full"
            );
        }
        for (id, oracle) in &self.oracles {
            ensure!(
                !oracle.target.trim().is_empty() && !oracle.target.starts_with('-'),
                "oracles.{id}: invalid target"
            );
            let check = self
                .checks
                .iter()
                .find(|c| c.id == oracle.check)
                .with_context(|| format!("oracles.{id}: unknown check {}", oracle.check))?;
            ensure!(
                check.kind == CheckKind::Command,
                "oracles.{id}: must reference command check"
            );
        }
        crate::lint::config::load(root, &relative(root, &self.paths.lint)?)
            .context("paths.lint")?;
        Ok(())
    }
}
