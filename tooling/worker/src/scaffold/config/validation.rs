use super::*;
use crate::lint::config::{globs, skill};
use std::{collections::HashSet, fs};

impl Config {
    pub(super) fn validate(&self, root: &Path) -> Result<()> {
        self.validate_structure(root)?;
        self.capabilities.validate(root, &self.checks)?;
        if self.capabilities.lint {
            crate::lint::config::load(root, &relative(root, &self.paths.lint)?)
                .context("paths.lint")?;
        }
        Ok(())
    }
    pub(super) fn validate_structure(&self, root: &Path) -> Result<()> {
        ensure!(self.version == 1, "version: supported schema is 1");
        ensure!(
            self.runtime == VERSION,
            "runtime: project pins {}, binary is {VERSION}; install the pinned binary",
            self.runtime
        );
        skill(root, &self.config_skill).context("config_skill")?;
        self.validate_paths(root)?;
        self.validate_commands(root)?;
        self.validate_checks(root)?;
        self.hooks.validate(root)?;
        self.validate_oracles()?;
        Ok(())
    }
    fn validate_paths(&self, root: &Path) -> Result<()> {
        for (label, value) in [
            ("service", &self.paths.service),
            ("memory", &self.paths.memory),
            ("skills", &self.paths.skills),
            ("lint", &self.paths.lint),
            ("runtime", &self.paths.runtime),
        ] {
            relative(root, value).with_context(|| format!("paths.{label}"))?;
        }
        ensure!(
            !self.paths.service.contains(['\n', '\r']) && !self.paths.service.contains("{{"),
            "paths.service cannot contain newlines or Just interpolation syntax"
        );
        ensure!(
            !self.paths.sources.is_empty(),
            "paths.sources cannot be empty"
        );
        globs(&self.paths.sources).context("paths.sources")?;
        ensure!(
            branch_name(&self.git.base),
            "git.base must name a valid branch"
        );
        ensure!(
            !self.git.prefix.is_empty()
                && branch_name(&format!("{}example", self.git.prefix))
                && !self.git.base.starts_with(&self.git.prefix),
            "git.prefix must form valid branches distinct from git.base"
        );
        Ok(())
    }
    fn validate_commands(&self, root: &Path) -> Result<()> {
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
        Ok(())
    }
    fn validate_oracles(&self) -> Result<()> {
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
        Ok(())
    }
    fn validate_checks(&self, root: &Path) -> Result<()> {
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
            check.validate_command(&self.commands)?;
        }
        Ok(())
    }
}
impl Check {
    fn validate_command(&self, commands: &BTreeMap<String, Command>) -> Result<()> {
        match (&self.kind, &self.command) {
            (CheckKind::Command, Some(command)) => {
                let spec = commands
                    .get(command)
                    .with_context(|| format!("checks.{}: unknown command {command}", self.id))?;
                ensure!(
                    !spec.argv.is_empty(),
                    "checks.{}: command needs an executable",
                    self.id
                );
            }
            (CheckKind::Command, None) => {
                anyhow::bail!("checks.{}: command is required", self.id)
            }
            (_, Some(_)) => {
                anyhow::bail!("checks.{}: builtin check cannot specify command", self.id)
            }
            _ => {}
        }
        Ok(())
    }
}
impl Hooks {
    fn validate(&self, root: &Path) -> Result<()> {
        for route in &self.routes {
            ensure!(!route.include.is_empty(), "hooks.routes: include required");
            globs(&route.include).context("hooks.routes.include")?;
            skill(root, &route.skill).context("hooks.routes.skill")?;
        }
        if let Some(path) = &self.discipline_skill {
            skill(root, path).context("hooks.discipline_skill")?;
        }
        if let Some(path) = &self.reminder {
            ensure!(
                self.discipline_skill.is_some(),
                "hooks.reminder requires hooks.discipline_skill"
            );
            validate_reminder(root, path)?;
        }
        Ok(())
    }
}
fn validate_reminder(root: &Path, path: &str) -> Result<()> {
    let source = fs::read_to_string(relative(root, path)?).context("hooks.reminder")?;
    let value: serde_json::Value = serde_json::from_str(&source).context("hooks.reminder JSON")?;
    let fields = value
        .as_object()
        .context("hooks.reminder must be an object")?;
    for field in fields.keys() {
        ensure!(
            [
                "attention_interval_tokens",
                "full_refresh_interval_tokens",
                "attention_message"
            ]
            .contains(&field.as_str()),
            "hooks.reminder: unknown field {field}"
        );
    }
    if let Some(message) = fields.get("attention_message") {
        ensure!(
            message.as_str().is_some_and(|text| !text.trim().is_empty()),
            "hooks.reminder.attention_message must be a nonempty string"
        );
    }
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
    Ok(())
}
