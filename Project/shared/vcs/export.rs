use super::{FileKind, Repository};
use crate::shared::fs::{self, Permissions, PermissionsExt};
use anyhow::{Result, ensure};
use std::{
    ffi::OsStr,
    os::unix::ffi::OsStrExt,
    path::{Component, Path},
};

pub(super) fn revision(repository: &Repository<'_>, revision: &str) -> Result<tempfile::TempDir> {
    let directory = tempfile::tempdir()?;
    let entries = repository.tree(revision)?;
    let mut links_after_contents = Vec::new();
    for (name, entry) in entries {
        validate_path(&name)?;
        ensure!(
            entry.kind != FileKind::Submodule,
            "revision export cannot include submodule {name}; export its repository separately"
        );
        let path = directory.path().join(&name);
        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("exported path has no parent"))?;
        fs::create_dir_all(parent)?;
        let bytes = repository.read(revision, &name)?;
        let symlink = entry.kind == FileKind::Symlink;
        if symlink {
            links_after_contents.push((path, bytes));
        } else {
            fs::write(&path, bytes)?;
            let executable = entry.kind == FileKind::Executable;
            let mode = if executable { 0o755 } else { 0o644 };
            fs::set_permissions(path, Permissions::from_mode(mode))?;
        }
    }
    for (path, target) in links_after_contents {
        fs::symlink(OsStr::from_bytes(&target), path)?;
    }
    Ok(directory)
}

pub(super) fn validate_path(name: &str) -> Result<()> {
    let normal = !name.is_empty()
        && !name.contains('\0')
        && name.split('/').all(|part| !matches!(part, "" | "." | ".."))
        && Path::new(name)
            .components()
            .all(|part| matches!(part, Component::Normal(value) if value != ".git"));
    ensure!(normal, "unsafe VCS export path: {name}");
    Ok(())
}
