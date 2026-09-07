use super::{Layout, Reviewer};
use anyhow::{Context, Result};
use serde_json::json;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(super) fn executable() -> Result<PathBuf> {
    if let Some(path) = env::var_os("REVIEW_CLAUDE_BIN") {
        return PathBuf::from(path)
            .canonicalize()
            .context("resolve REVIEW_CLAUDE_BIN");
    }
    let path = env::var_os("PATH").context("PATH is required to find claude")?;
    env::split_paths(&path)
        .map(|directory| directory.join("claude"))
        .find(|path| path.is_file())
        .context("claude is not installed; set REVIEW_CLAUDE_BIN to the native Linux executable")?
        .canonicalize()
        .context("resolve claude executable")
}

pub(super) fn prepare(role: &Path) -> Result<()> {
    let settings = json!({"hooks":{"Stop":[{"hooks":[{"type":"command",
        "command":"/review-bin/review-runner review-hook", "timeout":10}]}]}});
    fs::write(
        role.join("bin/settings.json"),
        serde_json::to_vec(&settings)?,
    )?;
    Ok(())
}

pub(super) fn command(command: &mut Command, layout: &Layout, reviewer: &Reviewer) {
    command
        .arg("--ro-bind")
        .arg(&layout.executable)
        .arg("/claude-cli")
        .args([
            "--setenv",
            "CLAUDE_CONFIG_DIR",
            "/claude",
            "/claude-cli",
            "--print",
            "--verbose",
            "--output-format",
            "stream-json",
            "--no-session-persistence",
            "--setting-sources",
            "",
            "--settings",
            "/review-bin/settings.json",
            "--strict-mcp-config",
            "--mcp-config",
            "{\"mcpServers\":{}}",
            "--dangerously-skip-permissions",
            "--model",
            &reviewer.model,
            "--effort",
            &reviewer.reasoning_effort,
        ]);
}
