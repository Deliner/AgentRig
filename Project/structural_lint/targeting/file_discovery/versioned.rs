use super::inventory::{Inventory, directories};
use super::unversioned::unversioned;
use crate::shared::fs_with_context as fs;
use anyhow::Result;
use globset::GlobSet;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub fn maintained_inventory(root: &Path, exclude: &GlobSet) -> Result<Inventory> {
    let Some(repository) = crate::shared::vcs::Repository::discover(root)? else {
        return unversioned(root, exclude);
    };
    let listed = repository.working_files()?;
    let directory_paths = repository.working_directories(&listed)?;
    let files = selected(root, exclude, listed);
    let mut directories = directories(&files);
    add_directories(root, exclude, directory_paths, &mut directories);
    Ok(Inventory { files, directories })
}

pub fn maintained_files(root: &Path, exclude: &GlobSet) -> Result<Vec<PathBuf>> {
    match crate::shared::vcs::Repository::discover(root)? {
        Some(repository) => Ok(selected(root, exclude, repository.working_files()?)),
        None => Ok(unversioned(root, exclude)?.files),
    }
}

fn selected(root: &Path, exclude: &GlobSet, listed: Vec<String>) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = listed.into_iter().map(PathBuf::from).collect();
    files.sort();
    files.dedup();
    files.retain(|path| {
        !exclude.is_match(path)
            && fs::symlink_metadata(root.join(path)).is_ok_and(|meta| meta.is_file())
    });
    files
}

fn add_directories(
    root: &Path,
    exclude: &GlobSet,
    paths: Vec<String>,
    output: &mut BTreeMap<PathBuf, BTreeSet<String>>,
) {
    output.entry(".".into()).or_default();
    for path in paths.into_iter().map(PathBuf::from) {
        let selected = !exclude.is_match(&path)
            && fs::symlink_metadata(root.join(&path)).is_ok_and(|meta| meta.is_dir());
        if selected {
            output.entry(path.clone()).or_default();
            for (parent, entries) in directories(&[path]) {
                output.entry(parent).or_default().extend(entries);
            }
        }
    }
}
