use super::super::{config, package::manifest};
use super::model::State;
use anyhow::{Context as _, Result, ensure};
use serde::Serialize;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub fn directory(root: &Path) -> Result<PathBuf> {
    let source = fs::read_to_string(root.join(config::FILE))?;
    let value: toml::Value = toml::from_str(&source)?;
    let runtime = value
        .get("paths")
        .and_then(|value| value.get("runtime"))
        .and_then(toml::Value::as_str)
        .context("paths.runtime required for upgrade recovery")?;
    config::relative(root, &format!("{runtime}/upgrade"))
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
pub fn json(path: &Path, value: &impl Serialize) -> Result<()> {
    atomic(path, &serde_json::to_vec_pretty(value)?, 0o600)
}
pub fn blob(directory: &Path, bytes: &[u8]) -> Result<String> {
    let hash = manifest::checksum(bytes);
    let path = directory.join("blobs").join(&hash);
    let absent = !path.exists();
    if absent {
        atomic(&path, bytes, 0o600)?;
    }
    Ok(hash)
}
pub fn payload(directory: &Path, hash: &str) -> Result<Vec<u8>> {
    ensure!(
        hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "invalid payload checksum"
    );
    let bytes = fs::read(directory.join("blobs").join(hash)).context("read upgrade payload")?;
    ensure!(
        manifest::checksum(&bytes) == hash,
        "upgrade payload changed: {hash}"
    );
    Ok(bytes)
}
pub fn replace(root: &Path, target: &State, directory: &Path) -> Result<()> {
    let path = config::relative(root, &target.resolved)?;
    match &target.sha256 {
        Some(hash) => atomic(
            &path,
            &payload(directory, hash)?,
            target.mode.context("file mode required")?,
        ),
        None => {
            let exists = path.exists();
            if exists {
                fs::remove_file(&path)?;
                fs::File::open(path.parent().context("file parent required")?)?.sync_all()?;
            }
            Ok(())
        }
    }
}
