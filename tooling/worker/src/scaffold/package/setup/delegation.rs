use super::{
    Config,
    registration::{setting, table},
};
use anyhow::{Result, ensure};
use std::{collections::BTreeSet, path::Path};
use toml_edit::{DocumentMut, Item, value};

pub fn configure(
    root: &Path,
    config: &Config,
    files: &super::Files,
    document: &mut DocumentMut,
) -> Result<()> {
    let Some(capability) = &config.capabilities.delegation else {
        return disabled(document);
    };
    let profiles: agentrig::delegate::config::Config =
        review_runner::config::yaml::decode(&super::source(root, files, &capability.config)?)?;
    table(&mut document["mcp_servers"], "mcp_servers")?;
    let server = &mut document["mcp_servers"]["worker_delegation"];
    table(server, "mcp_servers.worker_delegation")?;
    let mut args = toml_edit::Array::new();
    args.push("-c");
    args.push(super::super::adapters::mcp_command(
        &config.paths.service,
        "delegate",
        &super::super::adapters::generated(root, config)?.root_command,
    ));
    for (key, desired) in [
        ("enabled", value(true)),
        ("command", value("sh")),
        ("args", value(args)),
        ("tool_timeout_sec", value(60)),
        ("env_vars", value(environment(&profiles))),
    ] {
        setting(
            &mut server[key],
            desired,
            &format!("worker_delegation.{key}"),
        )?;
    }
    Ok(())
}

fn disabled(document: &DocumentMut) -> Result<()> {
    let server = document
        .get("mcp_servers")
        .and_then(|servers| servers.get("worker_delegation"));
    ensure!(
        server.is_none_or(|server| server.get("enabled").and_then(Item::as_bool) == Some(false)),
        "setup conflict: delegation is disabled; disable or remove mcp_servers.worker_delegation; existing settings preserved"
    );
    Ok(())
}

fn environment(config: &agentrig::delegate::config::Config) -> toml_edit::Array {
    let mut names = BTreeSet::from([
        "DELEGATE_CODEX_BIN",
        "WORKER_OWNER",
        "WORKER_PARENT_RUN",
        "CODEX_THREAD_ID",
        "CODEX_SESSION_ID",
    ]);
    for profile in config.profiles.values() {
        let claude = matches!(
            profile.frontend,
            agentrig::environment::Frontend::ClaudeCode
        );
        if claude {
            names.insert("DELEGATE_CLAUDE_BIN");
        }
        names.extend(profile.credentials.codex_auth_file_env.as_deref());
        names.extend(profile.credentials.env.values().map(String::as_str));
        for server in profile.environment.mcp_servers.values() {
            names.extend(server.env.values().map(String::as_str));
        }
    }
    names.into_iter().collect()
}
