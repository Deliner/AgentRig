use super::{FileKind, Repository};
use anyhow::{Result, ensure};
use std::{
    ffi::OsStr,
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Component, Path},
};

pub(super) fn revision(repository: &Repository<'_>, revision: &str) -> Result<tempfile::TempDir> {
    let directory = tempfile::tempdir()?;
    let entries = repository.tree(revision)?;
    let mut links = Vec::new();
    for (name, entry) in entries {
        validate_path(&name)?;
        ensure!(
            entry.kind != FileKind::Submodule,
            "revision export cannot include submodule {name}; export its repository separately"
        );
        let path = directory.path().join(&name);
        fs::create_dir_all(path.parent().expect("exported path has a parent"))?;
        let bytes = repository.read(revision, &name)?;
        match entry.kind {
            FileKind::Symlink => links.push((path, bytes)),
            FileKind::File | FileKind::Executable => {
                fs::write(&path, bytes)?;
                let mode = match entry.kind {
                    FileKind::Executable => 0o755,
                    _ => 0o644,
                };
                fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
            }
            FileKind::Submodule => unreachable!("rejected before reading"),
        }
    }
    // Create links last so tree entries cannot write through a link outside the export.
    for (path, target) in links {
        std::os::unix::fs::symlink(OsStr::from_bytes(&target), path)?;
    }
    Ok(directory)
}

pub(super) fn validate_path(name: &str) -> Result<()> {
    let normal = !name.is_empty()
        && !name.contains('\0')
        && name.split('/').all(|part| !matches!(part, "" | "." | ".."))
        && Path::new(name).components().all(
            |part| matches!(part, Component::Normal(value) if value != ".git" && value != ".hg"),
        );
    ensure!(normal, "unsafe VCS export path: {name}");
    Ok(())
}
