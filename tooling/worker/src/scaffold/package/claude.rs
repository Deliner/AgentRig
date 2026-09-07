use super::{Config, Files, adapters, config::Context};
use anyhow::{Context as _, Result, ensure};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub(super) fn bundle(files: &mut Files, config: &Config, root_command: &str) -> Result<()> {
    files.insert("CLAUDE.md".into(), b"@AGENTS.md\n".to_vec());
    files.insert(
        ".claude/settings.json".into(),
        adapters::registration(config, root_command)?,
    );
    files.insert(
        ".mcp.json".into(),
        serde_json::to_vec_pretty(&json!({"mcpServers": {}}))?,
    );
    Ok(())
}

pub(super) fn configure(root: &Path, config: &Config, files: &mut Files) -> Result<()> {
    let mut settings = document(files, ".claude/settings.json")?;
    ensure!(
        settings
            .get("disableAllHooks")
            .is_none_or(|value| value == false),
        "setup conflict: Claude disableAllHooks must be false; existing settings preserved"
    );
    let root_command = adapters::generated(root, config)?.root_command;
    let expected: Value = serde_json::from_slice(&adapters::registration(config, &root_command)?)?;
    hooks(&mut settings, &expected)?;
    if let Some(agent) = &config.agent {
        model(&mut settings, agent)?;
    }
    let mut mcp = document(files, ".mcp.json")?;
    let servers = object(&mut mcp, "mcpServers")?;
    capabilities(servers, config, &root_command)?;
    for name in config.environment.mcp_servers.keys() {
        server(
            servers,
            name,
            adapters::environment_command(&config.paths.service, "mcp", name, &root_command),
        )?;
    }
    files.insert(
        ".claude/settings.json".into(),
        serde_json::to_vec_pretty(&settings)?,
    );
    files.insert(".mcp.json".into(), serde_json::to_vec_pretty(&mcp)?);
    Ok(())
}

fn capabilities(
    servers: &mut serde_json::Map<String, Value>,
    config: &Config,
    root_command: &str,
) -> Result<()> {
    for (enabled, name, command) in [
        (
            config.capabilities.review.is_some(),
            "worker_review",
            "review",
        ),
        (
            config.capabilities.delegation.is_some(),
            "worker_delegation",
            "delegate",
        ),
    ] {
        if enabled {
            server(
                servers,
                name,
                adapters::mcp_command(&config.paths.service, command, root_command),
            )?;
        } else {
            ensure!(
                !servers.contains_key(name),
                "setup conflict: {name} is disabled; remove its MCP registration; original preserved"
            );
        }
    }
    Ok(())
}

fn model(settings: &mut Value, agent: &super::config::Agent) -> Result<()> {
    for (name, desired) in [
        ("model", &agent.model),
        ("effortLevel", &agent.reasoning_effort),
    ] {
        if let Some(desired) = desired {
            setting(settings.as_object_mut().unwrap(), name, json!(desired))?;
        }
    }
    if let Some(api) = &agent.api {
        let helper = format!(
            "printf '%s' \"${{{}:?missing agent.api.key_env reference}}\"",
            api.key_env
        );
        setting(
            settings.as_object_mut().unwrap(),
            "apiKeyHelper",
            json!(helper),
        )?;
        if let Some(url) = &api.base_url {
            setting(object(settings, "env")?, "ANTHROPIC_BASE_URL", json!(url))?;
        }
    }
    Ok(())
}

fn setting(
    settings: &mut serde_json::Map<String, Value>,
    name: &str,
    desired: Value,
) -> Result<()> {
    let current = settings.entry(name).or_insert_with(|| desired.clone());
    ensure!(
        *current == desired,
        "setup conflict: Claude {name}; existing setting preserved"
    );
    Ok(())
}

fn document(files: &Files, path: &str) -> Result<Value> {
    let value: Value = serde_json::from_slice(&files[path])
        .with_context(|| format!("setup conflict: invalid {path}"))?;
    ensure!(
        value.is_object(),
        "setup conflict: {path} must be an object"
    );
    Ok(value)
}

fn object<'a>(value: &'a mut Value, name: &str) -> Result<&'a mut serde_json::Map<String, Value>> {
    value
        .as_object_mut()
        .context("settings object required")?
        .entry(name)
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .with_context(|| format!("setup conflict: {name} must be an object"))
}

fn hooks(settings: &mut Value, expected: &Value) -> Result<()> {
    let hooks = object(settings, "hooks")?;
    for (event, groups) in expected["hooks"].as_object().unwrap() {
        let routes = hooks
            .entry(event)
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .with_context(|| format!("setup conflict: hooks.{event} must be an array"))?;
        for group in groups.as_array().unwrap() {
            let missing = !routes.contains(group);
            if missing {
                routes.push(group.clone());
            }
        }
    }
    Ok(())
}

fn server(servers: &mut serde_json::Map<String, Value>, name: &str, command: String) -> Result<()> {
    let desired = json!({"type": "stdio", "command": "sh", "args": ["-c", command]});
    let current = servers.entry(name).or_insert_with(|| desired.clone());
    ensure!(
        *current == desired,
        "setup conflict: mcpServers.{name}; existing setting preserved"
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
    println!(
        "Claude Code registration: {}",
        if registered {
            "registered"
        } else {
            "MISSING OR CHANGED; inspect .claude/settings.json"
        }
    );
    Ok(registered)
}

pub(super) fn report(files: &Files, config: &Config) -> Result<Value> {
    let mcp = document(files, ".mcp.json")?;
    let servers: serde_json::Map<String, Value> = mcp["mcpServers"]
        .as_object()
        .context("mcpServers object required")?
        .iter()
        .filter(|(name, _)| {
            (name.as_str() == "worker_review" && config.capabilities.review.is_some())
                || (name.as_str() == "worker_delegation"
                    && config.capabilities.delegation.is_some())
                || config.environment.mcp_servers.contains_key(*name)
        })
        .map(|(name, server)| (name.clone(), server.clone()))
        .collect();
    Ok(
        json!({"hooks": ".claude/settings.json", "instructions": "CLAUDE.md", "mcp_servers": servers}),
    )
}
