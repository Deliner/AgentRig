use super::{Config, Files, adapters};
use crate::project_config::Context;
use crate::shared::calling_agent::Agent;
use crate::shared::console;
use crate::shared::fs_with_context as fs;
use anyhow::{Context as _, Result};
use serde_json::Value;

pub(super) fn bundle(files: &mut Files, config: &Config, root_command: &str) -> Result<()> {
    // DECISION: D195
    let instructions = files
        .get("AGENTS.md")
        .context("CLAUDE.md copies AGENTS.md, which the bundle lacks")?
        .clone();
    files.insert("CLAUDE.md".into(), instructions);
    files.insert(
        ".claude/settings.json".into(),
        adapters::registration(config, root_command, Agent::ClaudeCode)?,
    );
    Ok(())
}

pub(super) fn registration(context: &Context) -> Result<bool> {
    let settings = fs::read(context.root.join(".claude/settings.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
    let enabled = settings.as_ref().is_some_and(|value| {
        value
            .get("disableAllHooks")
            .is_none_or(|value| value == false)
    });
    let registered =
        enabled && super::doctor::hooks_registered(context, settings.as_ref(), Agent::ClaudeCode)?;
    console::stdout(format!(
        "Claude Code registration: {}",
        if registered {
            "registered"
        } else {
            "MISSING OR CHANGED; inspect .claude/settings.json"
        }
    ));
    Ok(registered)
}
