use anyhow::{Result, bail};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
#[cfg(test)]
mod tests;

pub fn resolve(path: &Path) -> Result<PathBuf> {
    resolve_limited(path, 0)
}
fn resolve_limited(path: &Path, depth: u8) -> Result<PathBuf> {
    let cycle = depth > 32;
    if cycle {
        bail!("symlink cycle");
    }
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            other => result.push(other.as_os_str()),
        }
        let symlink = fs::symlink_metadata(&result).is_ok_and(|meta| meta.file_type().is_symlink());
        if symlink {
            let target = fs::read_link(&result)?;
            result.pop();
            result = resolve_limited(&result.join(target), depth + 1)?;
        }
    }
    Ok(result)
}
