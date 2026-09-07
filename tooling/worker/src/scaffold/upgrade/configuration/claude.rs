use super::{agents, config, hooks, managed_servers};
use anyhow::{Context, Result};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};

pub(super) fn merge(
    root: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
    current: &config::Config,
    configuration: &config::Config,
) -> Result<()> {
    let path = ".claude/settings.json";
    let exists = root.join(path).try_exists()?;
    if exists {
        let mut existing: Value = serde_json::from_slice(&fs::read(root.join(path))?)?;
        let desired = serde_json::from_slice(&files[path])?;
        hooks::merge(root, current, &mut existing, &desired)?;
        model(&mut existing, &desired, &agents(current, configuration))?;
        files.insert(path.into(), serde_json::to_vec_pretty(&existing)?);
    }
    mcp(root, files, &managed_servers(current, configuration))
}

fn model(existing: &mut Value, desired: &Value, agents: &[&config::Agent]) -> Result<()> {
    let fields = existing
        .as_object_mut()
        .context("Claude settings must be an object")?;
    for (field, owned) in [
        ("model", agents.iter().any(|agent| agent.model.is_some())),
        (
            "effortLevel",
            agents.iter().any(|agent| agent.reasoning_effort.is_some()),
        ),
        (
            "apiKeyHelper",
            agents.iter().any(|agent| agent.api.is_some()),
        ),
    ] {
        if owned {
            replace(fields, field, desired.get(field));
        }
    }
    let endpoint = agents
        .iter()
        .any(|agent| agent.api.as_ref().is_some_and(|api| api.base_url.is_some()));
    if endpoint {
        replace(
            hooks::object(existing, "env")?,
            "ANTHROPIC_BASE_URL",
            desired
                .get("env")
                .and_then(|env| env.get("ANTHROPIC_BASE_URL")),
        );
    }
    Ok(())
}

fn mcp(
    root: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
    names: &std::collections::BTreeSet<String>,
) -> Result<()> {
    let path = ".mcp.json";
    let missing = !root.join(path).try_exists()?;
    if missing {
        return Ok(());
    }
    let mut existing: Value = serde_json::from_slice(&fs::read(root.join(path))?)?;
    let desired: Value = serde_json::from_slice(&files[path])?;
    let servers = hooks::object(&mut existing, "mcpServers")?;
    for name in names {
        replace(servers, name, desired["mcpServers"].get(name));
    }
    files.insert(path.into(), serde_json::to_vec_pretty(&existing)?);
    Ok(())
}

fn replace(fields: &mut serde_json::Map<String, Value>, name: &str, desired: Option<&Value>) {
    match desired {
        Some(value) => {
            fields.insert(name.into(), value.clone());
        }
        None => {
            fields.remove(name);
        }
    }
}
