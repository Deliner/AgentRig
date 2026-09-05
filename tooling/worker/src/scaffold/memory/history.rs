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
    if listed.is_empty() {
        return Ok(());
    }
    let prior: Config =
        toml::from_str(&committed(git_root, FILE)?).context("committed worker.toml schema")?;
    let index = format!("{}/Decisions.md", prior.paths.memory);
    let rows = format::parse_table(&committed(git_root, &index)?, Path::new(&index), 'D', 3)?;
    let memory = context.path(&context.config.paths.memory)?;
    let current = format::table(&memory.join("Decisions.md"), 'D', 3)?;
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
        let detail = committed(git_root, &format!("{}/{}", prior.paths.memory, old.detail))?;
        ensure!(
            fs::read_to_string(memory.join(&row.detail))? == detail,
            "{}: committed decision detail cannot change; supersede with a new decision",
            old.id
        );
        let applications = format::links(&row.cells[2]);
        for link in format::links(&old.cells[2]) {
            ensure!(
                applications.contains(&link),
                "{}: committed application links are append-only",
                old.id
            );
        }
    }
    Ok(())
}
