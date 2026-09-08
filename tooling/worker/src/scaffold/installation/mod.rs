use super::{receipt as manifest, settings as config};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub resolved: String,
    pub sha256: Option<String>,
    pub mode: Option<u32>,
}

pub fn state(root: &Path, path: &str) -> Result<State> {
    let resolved = config::relative(root, path)?;
    let name = resolved.strip_prefix(root)?.to_string_lossy().into_owned();
    match fs::metadata(&resolved) {
        Ok(meta) => {
            ensure!(meta.is_file(), "upgrade target is not a file: {path}");
            Ok(State {
                resolved: name,
                sha256: Some(manifest::checksum(&fs::read(resolved)?)),
                mode: Some(meta.permissions().mode() & 0o777),
            })
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(State {
            resolved: name,
            sha256: None,
            mode: None,
        }),
        Err(error) => Err(error.into()),
    }
}
pub fn atomic(path: &Path, bytes: &[u8], mode: u32) -> Result<()> {
    let parent = path.parent().context("file parent required")?;
    fs::create_dir_all(parent)?;
    let mut pending = tempfile::NamedTempFile::new_in(parent)?;
    use std::io::Write;
    pending.write_all(bytes)?;
    pending
        .as_file()
        .set_permissions(fs::Permissions::from_mode(mode))?;
    pending.as_file().sync_all()?;
    pending.persist(path)?;
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}
