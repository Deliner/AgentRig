use super::{Files, config, manifest};
use crate::scaffold::upgrade::storage::{self, State};
use anyhow::{Result, ensure};
use manifest::{Manifest, Ownership};
use std::{collections::BTreeMap, fs, path::Path};

pub struct Installation {
    pub files: Files,
    before: BTreeMap<String, State>,
}
impl Installation {
    pub fn prepare(root: &Path, mut files: Files) -> Result<Self> {
        let old = read(root, manifest::PATH)?
            .map(|bytes| serde_json::from_slice::<Manifest>(&bytes))
            .transpose()?;
        let mut receipt: Manifest = serde_json::from_slice(&files[manifest::PATH])?;
        if let Some(old) = &old {
            ensure!(
                old.manifest_version == 1 && old.config_schema == 1,
                "unsupported installation manifest schema"
            );
            ensure!(
                old.package_version == receipt.package_version,
                "setup manifest version differs; use upgrade"
            );
            for (path, entry) in &old.files {
                receipt.files.entry(path.clone()).or_insert(entry.clone());
            }
            receipt.local = old.local.clone();
        }
        let mut before = BTreeMap::new();
        for (path, desired) in &mut files {
            let state = storage::state(root, path)?;
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
        files.insert(manifest::PATH.into(), serde_json::to_vec_pretty(&receipt)?);
        Ok(Self { files, before })
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
                let executable = manifest::executable(path);
                let default_mode = if executable { 0o755 } else { 0o644 };
                let mode = before.mode.unwrap_or(default_mode);
                storage::atomic(&config::relative(root, path)?, bytes, mode)?;
            }
        }
        Ok(())
    }
}
fn preserve(
    path: &str,
    desired: &mut Vec<u8>,
    actual: (&[u8], u32),
    receipts: (&Manifest, Option<&Manifest>),
) -> Result<()> {
    let (actual, mode) = actual;
    let receipt = path == manifest::PATH;
    if receipt {
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
