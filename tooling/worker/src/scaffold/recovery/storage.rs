pub use crate::scaffold::installation::{State, atomic, state};
use crate::scaffold::{receipt as manifest, settings as config};
use anyhow::{Context as _, Result, ensure};
use serde::Serialize;
use std::{fs, path::Path};

pub use super::directory;
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
