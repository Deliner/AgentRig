use super::{Config, Files, adapters};
use crate::project_config::Context;
use crate::shared::console;
use crate::shared::fs;
use anyhow::Result;
use serde_json::Value;

pub(super) fn bundle(files: &mut Files, config: &Config, root_command: &str) -> Result<()> {
    files.insert("CLAUDE.md".into(), b"@AGENTS.md\n".to_vec());
    files.insert(
        ".claude/settings.json".into(),
        adapters::registration(config, root_command)?,
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
    let registered = enabled && super::doctor::hooks_registered(context, settings.as_ref())?;
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
