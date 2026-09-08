pub(crate) mod model;
#[cfg(test)]
mod tests;
use super::{receipt, settings};
use anyhow::{Context as _, Result, ensure};
use model::{Journal, Plan};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const LEGACY_FILE: &str = "worker.toml";

pub fn journal(root: &Path) -> Result<Option<Journal>> {
    let present = directory(root)?.join("operation/journal.json").is_file();
    if present {
        Ok(Some(load(root)?.2))
    } else {
        Ok(None)
    }
}
pub fn active(root: &Path) -> Result<bool> {
    Ok(journal(root)?
        .is_some_and(|journal| !matches!(journal.phase.as_str(), "applied" | "rolled-back")))
}
pub fn configuration_pending(root: &Path) -> Result<bool> {
    Ok(!root.join(settings::FILE).is_file() && root.join(LEGACY_FILE).is_file() && active(root)?)
}
pub fn guard(root: &Path) -> Result<()> {
    ensure!(
        !active(root)?,
        "unfinished upgrade: use upgrade apply PLAN or upgrade rollback before delivery"
    );
    Ok(())
}
pub fn guidance(root: &Path) -> Result<String> {
    Ok(journal(root)?.map(|journal| format!("Upgrade operation: {}. Next action: {}. Use the new release executable with --root {} for recovery.",
        journal.phase, journal.next_action, root.display())).unwrap_or_default())
}

pub fn directory(root: &Path) -> Result<PathBuf> {
    let runtime = runtime(root)?;
    settings::relative(root, &format!("{runtime}/upgrade"))
}

fn runtime(root: &Path) -> Result<String> {
    let legacy = root.join(LEGACY_FILE).is_file();
    if legacy {
        let source = fs::read_to_string(root.join(LEGACY_FILE))?;
        let value: toml::Value = toml::from_str(&source)?;
        Ok(value
            .get("paths")
            .and_then(|paths| paths.get("runtime"))
            .and_then(toml::Value::as_str)
            .context("paths.runtime required for upgrade recovery")?
            .into())
    } else {
        Ok(settings::read(root)?.paths.runtime)
    }
}

pub fn load(root: &Path) -> Result<(PathBuf, Plan, Journal)> {
    let directory = directory(root)?.join("operation");
    let journal: Journal = serde_json::from_slice(&fs::read(directory.join("journal.json"))?)?;
    ensure!(journal.version == 1, "unsupported upgrade journal");
    let bytes = fs::read(directory.join("plan.json"))?;
    ensure!(
        receipt::checksum(&bytes) == journal.plan_sha256,
        "saved upgrade plan changed"
    );
    let plan: Plan = serde_json::from_slice(&bytes)?;
    ensure!(
        Path::new(&plan.project) == root,
        "operation belongs to another project"
    );
    Ok((directory, plan, journal))
}
