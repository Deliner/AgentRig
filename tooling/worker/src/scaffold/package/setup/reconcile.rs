use super::{Files, config};
use crate::scaffold::receipt as manifest;
use crate::scaffold::upgrade::storage::{self, State};
use anyhow::{Result, ensure};
use manifest::{Manifest, Ownership};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub struct Installation {
    pub files: Files,
    before: BTreeMap<String, State>,
    executable: BTreeSet<String>,
}
impl Installation {
    pub fn prepare(root: &Path, mut files: Files, receipt_path: &str) -> Result<Self> {
        let old = read(root, receipt_path)?
            .map(|bytes| serde_json::from_slice::<Manifest>(&bytes))
            .transpose()?;
        let mut receipt: Manifest = serde_json::from_slice(&files[receipt_path])?;
        if let Some(old) = &old {
            retain_receipt(&mut receipt, old)?;
        }
        let mut before = BTreeMap::new();
        for (path, desired) in &mut files {
            let state = storage::state(root, path)?;
            let is_receipt = path == receipt_path;
            if is_receipt {
                before.insert(path.clone(), state);
                continue;
            }
            if let Some(hash) = &state.sha256 {
                let actual = fs::read(root.join(&state.resolved))?;
                ensure!(
                    manifest::checksum(&actual) == *hash,
                    "setup input changed: {path}"
                );
                preserve(
                    path,
                    desired,
                    (&actual, state.mode.unwrap()),
                    (&receipt, old.as_ref()),
                )?;
            }
            before.insert(path.clone(), state);
        }
        files.insert(receipt_path.into(), serde_json::to_vec_pretty(&receipt)?);
        Ok(Self {
            files,
            before,
            executable: permissions(&receipt),
        })
    }
    pub fn apply(&self, root: &Path) -> Result<()> {
        for (path, before) in &self.before {
            ensure!(
                storage::state(root, path)? == *before,
                "setup input changed: {path}; rerun setup"
            );
        }
        for (path, bytes) in &self.files {
            let before = &self.before[path];
            let changed = before.sha256.as_deref() != Some(&manifest::checksum(bytes));
            if changed {
                storage::atomic(
                    &config::relative(root, path)?,
                    bytes,
                    mode(self.executable.contains(path), before),
                )?;
            }
        }
        Ok(())
    }

    pub fn changes(&self) -> Vec<serde_json::Value> {
        self.files
            .iter()
            .filter_map(|(path, bytes)| {
                let before = &self.before[path];
                let after = manifest::checksum(bytes);
                let unchanged = before.sha256.as_deref() == Some(&after);
                if unchanged {
                    return None;
                }
                let existing = before.sha256.is_some();
                Some(serde_json::json!({
                    "path": path,
                    "action": if existing { "update" } else { "create" },
                    "before_sha256": before.sha256,
                    "after_sha256": after,
                    "mode": mode(self.executable.contains(path), before),
                }))
            })
            .collect()
    }
}
fn mode(executable: bool, before: &State) -> u32 {
    let default = if executable { 0o755 } else { 0o644 };
    before.mode.unwrap_or(default)
}
fn permissions(receipt: &Manifest) -> BTreeSet<String> {
    receipt
        .files
        .iter()
        .filter(|(_, entry)| entry.executable)
        .map(|(path, _)| path.clone())
        .collect()
}
fn retain_receipt(receipt: &mut Manifest, old: &Manifest) -> Result<()> {
    ensure!(
        old.manifest_version == 1 && old.config_schema == 1,
        "unsupported installation manifest schema"
    );
    ensure!(
        old.package_version == receipt.package_version,
        "setup manifest version differs; use upgrade"
    );
    for (path, entry) in &old.files {
        let retained = old.local.contains_key(path)
            || matches!(
                entry.ownership,
                Ownership::Configuration | Ownership::Memory
            );
        if retained {
            receipt.files.insert(path.clone(), entry.clone());
        } else {
            receipt.files.entry(path.clone()).or_insert(entry.clone());
        }
    }
    receipt.local = old.local.clone();
    Ok(())
}
fn preserve(
    path: &str,
    desired: &mut Vec<u8>,
    actual: (&[u8], u32),
    receipts: (&Manifest, Option<&Manifest>),
) -> Result<()> {
    let (actual, mode) = actual;
    let approved = receipts
        .1
        .and_then(|old| old.local.get(path))
        .and_then(Option::as_ref);
    if let Some(hash) = approved {
        ensure!(
            manifest::checksum(actual) == *hash,
            "setup conflict: {path}; approved local contents changed"
        );
        *desired = actual.to_vec();
        return Ok(());
    }
    let ownership = &receipts.0.files[path].ownership;
    let settings = matches!(ownership, Ownership::Configuration | Ownership::Memory);
    if settings {
        *desired = actual.to_vec();
        return Ok(());
    }
    let stock = receipts
        .1
        .and_then(|old| old.files.get(path))
        .is_some_and(|entry| {
            entry.sha256 == manifest::checksum(actual) && entry.executable == (mode & 0o111 != 0)
        });
    ensure!(
        actual == desired || stock,
        "setup conflict: {path}; local contents preserved; reconcile with the shipped adapter/asset before retrying"
    );
    Ok(())
}
fn read(root: &Path, path: &str) -> Result<Option<Vec<u8>>> {
    match fs::read(config::relative(root, path)?) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
