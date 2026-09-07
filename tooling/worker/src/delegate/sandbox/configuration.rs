use super::{Layout, Profile};
use crate::environment::resolve_references as resolve;
use anyhow::{Context, Result};
use serde_json::json;
use std::{env, fs, os::unix::fs::PermissionsExt, process::Command};

pub(super) fn write(layout: &Layout, profile: &Profile) -> Result<()> {
    fs::set_permissions(&layout.private, fs::Permissions::from_mode(0o700))?;
    let claude = matches!(profile.frontend, super::Frontend::ClaudeCode);
    if claude {
        return super::claude::configure(layout, profile);
    }
    let mut servers = serde_json::Map::new();
    for (name, server) in &profile.environment.mcp_servers {
        servers.insert(
            name.clone(),
            json!({"command":format!("/tools/{}",server.program),
            "args":server.args,"env":resolve(&server.env)?}),
        );
    }
    let config = json!({"model":profile.model,"model_reasoning_effort":profile.reasoning_effort,
    "features":{"hooks":!profile.environment.hooks.is_empty()},"mcp_servers":servers,
    "hooks":crate::environment::hooks::configuration(&profile.environment.hooks, |_, hook| {
        shell_words::join(std::iter::once(format!("/tools/{}", hook.program)).chain(hook.args.clone()))
    })});
    fs::write(
        layout.private.join("codex/config.toml"),
        toml::to_string(&config)?,
    )?;
    if let Some(reference) = &profile.credentials.codex_auth_file_env {
        let source = env::var_os(reference)
            .with_context(|| format!("missing credential reference {reference}"))?;
        let target = layout.private.join("codex/auth.json");
        fs::copy(source, &target).context("copy isolated Codex authentication")?;
        fs::set_permissions(target, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub(super) fn environment(command: &mut Command, profile: &Profile) -> Result<()> {
    command.envs(resolve(&profile.credentials.env)?);
    command
        .env("HOME", "/home/delegate")
        .env("PATH", "/tools:/bin")
        .env("SHELL", "/bin/bash");
    match profile.frontend {
        super::Frontend::Codex => command.env("CODEX_HOME", "/codex"),
        super::Frontend::ClaudeCode => command.env("CLAUDE_CONFIG_DIR", "/claude"),
    };
    Ok(())
}
