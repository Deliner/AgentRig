use super::{Config, Files};
use anyhow::{Context, Result, ensure};
use std::path::Path;
use toml_edit::{DocumentMut, Item, value};

pub fn configure(root: &Path, config: &Config, files: &mut Files) -> Result<()> {
    config
        .vcs
        .backend
        .source(root)
        .validate_registration(&config.paths.service_path("hooks"))?;
    let claude = matches!(config.frontend, super::config::Frontend::ClaudeCode);
    if claude {
        return super::super::claude::configure(root, config, files);
    }
    codex(root, config, files)
}

fn codex(root: &Path, config: &Config, files: &mut Files) -> Result<()> {
    let mut document: DocumentMut = std::str::from_utf8(&files[".codex/config.toml"])?.parse()?;
    table(&mut document["features"], "features")?;
    setting(
        &mut document["features"]["hooks"],
        value(true),
        "features.hooks",
    )?;
    super::delegation::configure(root, config, files, &mut document)?;
    super::environment::configure(root, config, &mut document)?;
    if let Some(review) = &config.capabilities.review {
        let review = review_configuration(root, files, &review.config)?;
        table(&mut document["mcp_servers"], "mcp_servers")?;
        mcp(
            &mut document["mcp_servers"]["worker_review"],
            &review,
            &config.paths.service,
            &super::super::adapters::generated(root, config)?.root_command,
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
fn review_configuration(
    root: &Path,
    files: &Files,
    path: &str,
) -> Result<review_runner::config::Config> {
    let source = super::source(root, files, path)?;
    review_runner::config::yaml::decode(&source)
}
fn mcp(
    server: &mut Item,
    review: &review_runner::config::Config,
    service: &str,
    root_command: &str,
) -> Result<()> {
    table(server, "mcp_servers.worker_review")?;
    setting(&mut server["enabled"], value(true), "worker_review.enabled")?;
    setting(&mut server["command"], value("sh"), "worker_review.command")?;
    let mut args = toml_edit::Array::new();
    args.push("-c");
    args.push(super::super::adapters::mcp_command(
        service,
        "review",
        root_command,
    ));
    setting(&mut server["args"], value(args), "worker_review.args")?;
    let timeout = i64::try_from(review.runner.timeout_seconds)?
        .checked_add(60)
        .context("review timeout too large for MCP")?;
    setting(
        &mut server["tool_timeout_sec"],
        value(timeout),
        "worker_review.tool_timeout_sec",
    )?;
    setting(
        &mut server["env_vars"],
        value(review_environment(review)),
        "worker_review.env_vars",
    )?;
    Ok(())
}
fn review_environment(review: &review_runner::config::Config) -> toml_edit::Array {
    let mut names = std::collections::BTreeSet::from(["CODEX_HOME", "REVIEW_CODEX_BIN"]);
    for reviewer in review.reviewers.values() {
        let claude = reviewer.frontend == "claude-code";
        if claude {
            names.insert("REVIEW_CLAUDE_BIN");
        }
        names.extend(reviewer.credentials.codex_auth_file_env.as_deref());
        names.extend(reviewer.credentials.env.values().map(String::as_str));
    }
    names.into_iter().collect()
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
