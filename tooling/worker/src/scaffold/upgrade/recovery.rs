use super::{model::Journal, operation::Operation, storage};
use anyhow::{Result, ensure};
use std::path::Path;

pub fn journal(root: &Path) -> Result<Option<Journal>> {
    let present = storage::directory(root)?
        .join("operation/journal.json")
        .is_file();
    if present {
        Ok(Some(Operation::open(root)?.journal))
    } else {
        Ok(None)
    }
}
pub fn active(root: &Path) -> Result<bool> {
    Ok(journal(root)?
        .is_some_and(|journal| !matches!(journal.phase.as_str(), "applied" | "rolled-back")))
}
pub fn configuration_pending(root: &Path) -> Result<bool> {
    Ok(!root.join(crate::scaffold::config::FILE).is_file()
        && root.join(super::migration::LEGACY_FILE).is_file()
        && active(root)?)
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
