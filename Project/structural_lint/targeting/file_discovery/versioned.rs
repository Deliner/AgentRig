use super::unversioned::unversioned;
use super::{Inventory, directories};
use crate::shared::fs_with_context as fs;
use anyhow::Result;
use globset::GlobSet;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub fn collect(root: &Path, exclude: &GlobSet) -> Result<Inventory> {
    let (files, directory_paths) =
        if let Some(repository) = crate::shared::vcs::Repository::discover(root)? {
            let files = repository.working_files()?;
            let directories = repository.working_directories(&files)?;
            (files, directories)
        } else {
            return unversioned(root, exclude);
        };
    let mut files: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
    files.sort();
    files.dedup();
    files.retain(|path| {
        !exclude.is_match(path)
            && fs::symlink_metadata(root.join(path)).is_ok_and(|meta| meta.is_file())
    });
    let mut directories = directories(&files);
    add_directories(root, exclude, directory_paths, &mut directories);
    Ok(Inventory { files, directories })
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
