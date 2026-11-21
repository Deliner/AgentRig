use super::repository::Repository;
use super::tree_entry::{Entry, FileKind};
use crate::shared::fs_with_context::{self as fs, Permissions, PermissionsExt};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
};

impl Repository<'_> {
    pub fn export_revision(&self, reference: &str) -> Result<tempfile::TempDir> {
        let resolved = self.resolve(reference)?;
        revision(self, &resolved)
    }
}

struct Exported {
    path: PathBuf,
    mode: Option<u32>,
    object: String,
}

fn revision(repository: &Repository<'_>, revision: &str) -> Result<tempfile::TempDir> {
    let directory = tempfile::tempdir()?;
    let files = planned(repository.tree(revision)?, directory.path())?;
    let objects: Vec<&str> = files.iter().map(|file| file.object.as_str()).collect();
    let mut pending = files.iter();
    let mut links_after_contents = Vec::new();
    repository.blobs(&objects, |bytes| {
        let file = pending
            .next()
            .context("Git returned an unrequested object")?;
        match file.mode {
            Some(mode) => {
                fs::write(&file.path, bytes)?;
                fs::set_permissions(&file.path, Permissions::from_mode(mode))?;
            }
            None => links_after_contents.push((file.path.clone(), bytes)),
        }
        Ok(())
    })?;
    for (path, target) in links_after_contents {
        fs::symlink(OsStr::from_bytes(&target), path)?;
    }
    Ok(directory)
}

fn planned(entries: BTreeMap<String, Entry>, directory: &Path) -> Result<Vec<Exported>> {
    let mut files = Vec::new();
    for (name, entry) in entries {
        validate_path(&name)?;
        let mode = match entry.kind {
            FileKind::Submodule => bail!(
                "revision export cannot include submodule {name}; export its repository separately"
            ),
            FileKind::Symlink => None,
            FileKind::Executable => Some(0o755),
            FileKind::File => Some(0o644),
        };
        let path = directory.join(&name);
        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("exported path has no parent"))?;
        fs::create_dir_all(parent)?;
        let object = entry.object;
        files.push(Exported { path, mode, object });
    }
    Ok(files)
}

pub(super) fn validate_path(name: &str) -> Result<()> {
    let present = !name.is_empty() && !name.contains('\0');
    let plain_segments = name.split('/').all(|part| !matches!(part, "" | "." | ".."));
    let normal_components = Path::new(name)
        .components()
        // DECISION: D156 variant-equality
        .all(|part| matches!(part, Component::Normal(value) if value != ".git"));
    let normal = present && plain_segments && normal_components;
    ensure!(normal, "unsafe VCS export path: {name}");
    Ok(())
}
