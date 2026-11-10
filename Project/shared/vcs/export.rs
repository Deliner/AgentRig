use super::{FileKind, Repository};
use crate::shared::fs::{self, Permissions, PermissionsExt};
use anyhow::{Result, bail, ensure};
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
        let file_mode = match entry.kind {
            FileKind::Submodule => bail!(
                "revision export cannot include submodule {name}; export its repository separately"
            ),
            FileKind::Symlink => None,
            FileKind::Executable => Some(0o755),
            FileKind::File => Some(0o644),
        };
        let path = directory.path().join(&name);
        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("exported path has no parent"))?;
        fs::create_dir_all(parent)?;
        let bytes = repository.read(revision, &name)?;
        match file_mode {
            Some(mode) => {
                fs::write(&path, bytes)?;
                fs::set_permissions(path, Permissions::from_mode(mode))?;
            }
            None => links_after_contents.push((path, bytes)),
        }
    }
    for (path, target) in links_after_contents {
        fs::symlink(OsStr::from_bytes(&target), path)?;
    }
    Ok(directory)
}

pub(super) fn validate_path(name: &str) -> Result<()> {
    let present = !name.is_empty() && !name.contains('\0');
    let plain_segments = name.split('/').all(|part| !matches!(part, "" | "." | ".."));
    let normal_components = Path::new(name)
        .components()
        // DECISION: D156
        .all(|part| matches!(part, Component::Normal(value) if value != ".git"));
    let normal = present && plain_segments && normal_components;
    ensure!(normal, "unsafe VCS export path: {name}");
    Ok(())
}
