use anyhow::Result;
use globset::GlobSet;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

// DECISION: D016
pub struct Inventory {
    pub files: Vec<PathBuf>,
    pub directories: BTreeMap<PathBuf, BTreeSet<String>>,
}
impl Inventory {
    pub fn file_entries(&self, directory: &Path) -> usize {
        let root = directory == Path::new(".");
        let prefix = if root { Path::new("") } else { directory };
        self.files
            .iter()
            .filter_map(|file| file.strip_prefix(prefix).ok()?.components().next())
            .collect::<BTreeSet<_>>()
            .len()
    }
}
pub fn collect(root: &Path, exclude: &GlobSet) -> Result<Inventory> {
    collect_source(root, exclude, None)
}

pub fn collect_source(
    root: &Path,
    exclude: &GlobSet,
    source: Option<&review_runner::vcs::Source<'_>>,
) -> Result<Inventory> {
    let (files, directory_paths) = if let Some(source) = source {
        let files = source.working_files()?;
        let directories = source.working_directories(&files)?;
        (files, directories)
    } else if let Some(repository) = review_runner::vcs::Repository::discover(root)? {
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
fn directories(files: &[PathBuf]) -> BTreeMap<PathBuf, BTreeSet<String>> {
    let mut directories: BTreeMap<PathBuf, BTreeSet<String>> = BTreeMap::new();
    for path in files {
        let mut child = path.as_path();
        while let Some(parent) = child.parent() {
            let root_entry = parent.as_os_str().is_empty();
            let directory = if root_entry {
                PathBuf::from(".")
            } else {
                parent.into()
            };
            directories
                .entry(directory)
                .or_default()
                .insert(child.file_name().unwrap().to_string_lossy().into_owned());
            child = parent;
        }
    }
    directories
}
fn unversioned(root: &Path, exclude: &GlobSet) -> Result<Inventory> {
    let mut inventory = Inventory {
        files: Vec::new(),
        directories: BTreeMap::new(),
    };
    walk(root, Path::new(""), exclude, &mut inventory)?;
    inventory.files.sort();
    for (directory, entries) in directories(&inventory.files) {
        inventory
            .directories
            .entry(directory)
            .or_default()
            .extend(entries);
    }
    Ok(inventory)
}

fn walk(root: &Path, relative: &Path, exclude: &GlobSet, inventory: &mut Inventory) -> Result<()> {
    let at_root = relative.as_os_str().is_empty();
    let parent = if at_root { Path::new(".") } else { relative };
    inventory.directories.entry(parent.into()).or_default();
    for entry in fs::read_dir(root.join(relative))? {
        let entry = entry?;
        let path = relative.join(entry.file_name());
        let ignored = exclude.is_match(&path) || entry.file_type()?.is_symlink();
        if ignored {
            continue;
        }
        let kind = entry.file_type()?;
        let directory = kind.is_dir();
        let file = kind.is_file();
        if directory {
            inventory
                .directories
                .entry(parent.into())
                .or_default()
                .insert(entry.file_name().to_string_lossy().into_owned());
            walk(root, &path, exclude, inventory)?;
        } else if file {
            inventory.files.push(path);
        }
    }
    Ok(())
}
