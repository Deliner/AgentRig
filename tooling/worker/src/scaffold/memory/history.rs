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
    let unborn = crate::util::git(git_root, &["rev-parse", "--verify", "HEAD"]).is_err();
    if unborn {
        return Ok(());
    }
    let prior_memory = prior_memory(context, git_root)?;
    let index = format!("{}/Decisions.md", prior_memory);
    let no_prior_memory =
        crate::util::git(git_root, &["ls-tree", "HEAD", "--", &index])?.is_empty();
    if no_prior_memory {
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
        let detail = committed(git_root, &format!("{}/{}", prior_memory, old.detail))?;
        preserve(
            old,
            row,
            &detail,
            &fs::read_to_string(memory.join(&row.detail))?,
        )?;
    }
    Ok(())
}

fn prior_memory(context: &Context, git_root: &Path) -> Result<String> {
    let current = !crate::util::git(git_root, &["ls-tree", "HEAD", "--", FILE])?.is_empty();
    if current {
        let prior: Config = review_runner::config::yaml::decode(&committed(git_root, FILE)?)
            .context("committed agentrig.yaml schema")?;
        return Ok(prior.paths.memory);
    }
    let legacy = super::super::upgrade::migration::LEGACY_FILE;
    let migrated = !crate::util::git(git_root, &["ls-tree", "HEAD", "--", legacy])?.is_empty();
    if migrated {
        return super::super::upgrade::migration::historical_memory(&committed(git_root, legacy)?);
    }
    Ok(context.config.paths.memory.clone())
}

fn preserve(
    old: format::Row,
    row: &format::Row,
    prior_detail: &str,
    current_detail: &str,
) -> Result<()> {
    ensure!(
        row.detail == old.detail && row.cells[1] == old.cells[1],
        "{}: committed decision identity cannot change",
        old.id
    );
    ensure!(
        current_detail == prior_detail,
        "{}: committed decision detail cannot change; supersede with a new decision",
        old.id
    );
    Ok(())
}
