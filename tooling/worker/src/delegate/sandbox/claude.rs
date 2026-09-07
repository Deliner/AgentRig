use super::{Layout, Profile};
use crate::environment::resolve_references;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{fs, io::Write, process::Command};

pub(super) fn configure(layout: &Layout, profile: &Profile) -> Result<()> {
    let hooks = crate::environment::hooks::configuration(&profile.environment.hooks, |_, hook| {
        shell_words::join(
            std::iter::once(format!("/tools/{}", hook.program)).chain(hook.args.clone()),
        )
    });
    let settings = json!({"hooks":hooks});
    fs::write(
        layout.private.join("claude/settings.json"),
        serde_json::to_vec(&settings)?,
    )?;
    let mut servers = serde_json::Map::new();
    for (name, server) in &profile.environment.mcp_servers {
        servers.insert(
            name.clone(),
            json!({"command":format!("/tools/{}", server.program),
            "args":server.args, "env":resolve_references(&server.env)?}),
        );
    }
    fs::write(
        layout.private.join("claude/mcp.json"),
        serde_json::to_vec(&json!({"mcpServers":servers}))?,
    )?;
    Ok(())
}

pub(super) fn executor(command: &mut Command, layout: &Layout, profile: &Profile) -> Result<()> {
    let schema = fs::read_to_string(layout.input.join("schema.json"))?;
    command
        .args([
            "/claude-cli",
            "--print",
            "--output-format",
            "json",
            "--json-schema",
            &schema,
            "--no-session-persistence",
            "--setting-sources",
            "",
            "--settings",
            "/claude/settings.json",
            "--strict-mcp-config",
            "--mcp-config",
            "/claude/mcp.json",
            "--dangerously-skip-permissions",
            "--model",
            &profile.model,
            "--effort",
            &profile.reasoning_effort,
        ])
        .arg(super::instructions(profile));
    Ok(())
}

pub(super) fn response(layout: &Layout, output: &[u8]) -> Result<()> {
    let envelope: Value = serde_json::from_slice(output).context("Claude response must be JSON")?;
    ensure!(
        envelope["is_error"] == false,
        "Claude reported an unsuccessful result"
    );
    let value = envelope
        .get("structured_output")
        .context("Claude response lacks structured_output")?;
    // The model can create arbitrary paths in /work; never follow a model-created symlink.
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(layout.private.join("work/result.json"))
        .context("result.json is reserved for the validated client response")?;
    file.write_all(&serde_json::to_vec(value)?)?;
    Ok(())
}
