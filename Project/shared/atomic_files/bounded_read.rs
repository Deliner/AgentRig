use crate::shared::fs_with_context::{self as fs, OpenOptions, OpenOptionsExt};
use anyhow::{Result, ensure};
use std::{io::Read, path::Path};

pub fn read_regular(path: &Path, max_bytes: u64) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_file() && metadata.len() <= max_bytes,
        "response must be a regular file of at most {max_bytes} bytes"
    );
    let file = fs::open_with(
        OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK),
        path,
    )?;
    ensure!(
        fs::file_metadata(&file, path)?.is_file(),
        "response is not a regular file"
    );
    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    ensure!(
        u64::try_from(bytes.len())? <= max_bytes,
        "response exceeds size limit"
    );
    Ok(bytes)
}
