use super::{Config, Files};
use anyhow::{Context, Result, ensure};
use std::path::Path;
use toml_edit::{DocumentMut, Item, value};

pub fn configure(root: &Path, config: &Config, files: &mut Files) -> Result<()> {
    if let Some(repository) = config.vcs.backend.repository(root)? {
        repository.validate_registration(&config.paths.service_path("hooks"))?;
    }
    let mut document: DocumentMut = std::str::from_utf8(&files[".codex/config.toml"])?.parse()?;
    table(&mut document["features"], "features")?;
    setting(
        &mut document["features"]["hooks"],
        value(true),
        "features.hooks",
    )?;
    super::delegation::configure(root, config, files, &mut document)?;
    super::environment::configure(config, &mut document)?;
    if let Some(review) = &config.capabilities.review {
        let timeout = review_timeout(root, files, &review.config)?;
        table(&mut document["mcp_servers"], "mcp_servers")?;
        mcp(
            &mut document["mcp_servers"]["worker_review"],
            timeout,
            &config.paths.service,
            config.vcs.backend,
        )?;
    } else {
        let server = document
            .get("mcp_servers")
            .and_then(|servers| servers.get("worker_review"));
        ensure!(
            server
                .is_none_or(|server| server.get("enabled").and_then(Item::as_bool) == Some(false)),
            "setup conflict: review is disabled in agentrig.yaml; disable or remove mcp_servers.worker_review; existing settings preserved"
        );
    }
    files.insert(
        ".codex/config.toml".into(),
        document.to_string().into_bytes(),
    );
    Ok(())
}
fn review_timeout(root: &Path, files: &Files, path: &str) -> Result<u64> {
    let source = super::source(root, files, path)?;
    let review: review_runner::config::Config = review_runner::config::yaml::decode(&source)?;
    Ok(review.runner.timeout_seconds)
}
fn mcp(
    server: &mut Item,
    timeout: u64,
    service: &str,
    backend: review_runner::vcs::Kind,
) -> Result<()> {
    table(server, "mcp_servers.worker_review")?;
    setting(&mut server["enabled"], value(true), "worker_review.enabled")?;
    setting(&mut server["command"], value("sh"), "worker_review.command")?;
    let mut args = toml_edit::Array::new();
    args.push("-c");
    args.push(super::super::adapters::mcp_command(
        service, "review", backend,
    ));
    setting(&mut server["args"], value(args), "worker_review.args")?;
    let timeout = i64::try_from(timeout)?
        .checked_add(60)
        .context("review timeout too large for MCP")?;
    setting(
        &mut server["tool_timeout_sec"],
        value(timeout),
        "worker_review.tool_timeout_sec",
    )?;
    let mut forwarded = toml_edit::Array::new();
    forwarded.push("CODEX_HOME");
    forwarded.push("REVIEW_CODEX_BIN");
    setting(
        &mut server["env_vars"],
        value(forwarded),
        "worker_review.env_vars",
    )?;
    Ok(())
}
pub(super) fn table(item: &mut Item, name: &str) -> Result<()> {
    ensure!(
        item.is_none() || item.is_table_like(),
        "setup conflict: {name} must be a table; original preserved"
    );
    let absent = item.is_none();
    if absent {
        let mut table = toml_edit::Table::new();
        table.set_implicit(true);
        *item = Item::Table(table);
    }
    Ok(())
}
pub(super) fn setting(current: &mut Item, desired: Item, name: &str) -> Result<()> {
    let absent = current.is_none();
    if absent {
        *current = desired;
    } else {
        let actual: toml::Value = toml::from_str(&format!("value={current}"))?;
        let expected: toml::Value = toml::from_str(&format!("value={desired}"))?;
        ensure!(
            actual == expected,
            "setup conflict: {name}; existing MCP/hook setting preserved"
        );
    }
    Ok(())
}
