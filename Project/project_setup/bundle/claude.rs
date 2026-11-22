use super::adapters;
use super::contents::Files;
use crate::project_config::{Config, Context};
use crate::shared::calling_agent::Agent;
use crate::shared::console;
use crate::shared::fs_with_context as fs;
use anyhow::{Context as _, Result};
use serde_json::{Value, json};

// DECISION: D231
const MARKETPLACE: &str = "agentrig";
const MODS: [(&str, &str); 4] = [
    (
        ".claude-plugin/marketplace.json",
        include_str!("../../shared/mods/.claude-plugin/marketplace.json"),
    ),
    (
        "self-compact/.claude-plugin/plugin.json",
        include_str!("../../shared/mods/self-compact/.claude-plugin/plugin.json"),
    ),
    (
        "self-compact/hooks/hooks.json",
        include_str!("../../shared/mods/self-compact/hooks/hooks.json"),
    ),
    (
        "self-compact/hooks/register.ts",
        include_str!("../../shared/mods/self-compact/hooks/register.ts"),
    ),
];

pub(super) fn bundle(files: &mut Files, config: &Config, root_command: &str) -> Result<()> {
    // DECISION: D195
    let instructions = files
        .get("AGENTS.md")
        .context("CLAUDE.md copies AGENTS.md, which the bundle lacks")?
        .clone();
    files.insert("CLAUDE.md".into(), instructions);
    let mut settings = adapters::settings(config, root_command, Agent::ClaudeCode);
    install_mods(files, config, &mut settings);
    files.insert(
        ".claude/settings.json".into(),
        serde_json::to_vec_pretty(&settings)?,
    );
    Ok(())
}

// DECISION: D231
fn install_mods(files: &mut Files, config: &Config, settings: &mut Value) {
    let directory = config.paths.service_path("mods");
    for (path, source) in MODS {
        files.insert(format!("{directory}/{path}"), source.as_bytes().to_vec());
    }
    settings["extraKnownMarketplaces"][MARKETPLACE] =
        json!({"source": {"source": "directory", "path": directory}});
    settings["enabledPlugins"][format!("self-compact@{MARKETPLACE}")] = json!(true);
}

pub fn registration(context: &Context) -> Result<bool> {
    let settings = fs::read(context.root.join(".claude/settings.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
    let enabled = settings.as_ref().is_some_and(|value| {
        value
            .get("disableAllHooks")
            .is_none_or(|value| value == false)
    });
    let registered = enabled
        && adapters::hooks_registered(&context.config, settings.as_ref(), Agent::ClaudeCode)?;
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
