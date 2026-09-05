// DECISION: D020
use super::super::config::{Config, Context, FILE};
use super::format;
use anyhow::{Context as _, Result, ensure};
use std::{fs, path::Path, process::Command};

fn committed(root: &Path, path: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["show", &format!("HEAD:{path}")])
        .current_dir(root)
        .output()?;
    ensure!(
        output.status.success(),
        "cannot read committed memory {path}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}
pub fn check(context: &Context, git_root: &Path) -> Result<()> {
    // A standalone tree or an unborn repository has no committed memory baseline.
    if crate::util::git(git_root, &["rev-parse", "--verify", "HEAD"]).is_err() {
        return Ok(());
    }
    let listed = crate::util::git(git_root, &["ls-tree", "HEAD", "--", FILE])?;
    let prior_memory = if listed.is_empty() {
        context.config.paths.memory.clone()
    } else {
        let prior: Config =
            toml::from_str(&committed(git_root, FILE)?).context("committed worker.toml schema")?;
        prior.paths.memory
    };
    let index = format!("{}/Decisions.md", prior_memory);
    if crate::util::git(git_root, &["ls-tree", "HEAD", "--", &index])?.is_empty() {
        return Ok(());
    }
    let rows = format::parse_table(&committed(git_root, &index)?, Path::new(&index), 'D')?;
    let memory = context.path(&context.config.paths.memory)?;
    let current = format::table(&memory.join("Decisions.md"), 'D')?;
    for old in rows {
        let row = current
            .iter()
            .find(|row| row.id == old.id)
            .with_context(|| format!("{}: committed decision cannot be removed", old.id))?;
        ensure!(
            row.detail == old.detail && row.cells[1] == old.cells[1],
            "{}: committed decision identity cannot change",
            old.id
        );
        let detail = committed(git_root, &format!("{}/{}", prior_memory, old.detail))?;
        ensure!(
            fs::read_to_string(memory.join(&row.detail))? == detail,
            "{}: committed decision detail cannot change; supersede with a new decision",
            old.id
        );
    }
    Ok(())
}
