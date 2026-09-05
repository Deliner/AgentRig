use anyhow::Result;
use globset::GlobSet;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

// DECISION: D016
pub struct Inventory {
    pub files: Vec<PathBuf>,
    pub directories: BTreeMap<PathBuf, BTreeSet<String>>,
}
pub fn collect(root: &Path, exclude: &GlobSet) -> Result<Inventory> {
    let git_repository = root.join(".git").exists();
    let mut files = if git_repository {
        git_files(root)?
    } else {
        let mut files = Vec::new();
        walk(root, Path::new(""), exclude, &mut files)?;
        files
    };
    files.sort();
    files.dedup();
    files.retain(|path| {
        !exclude.is_match(path)
            && fs::symlink_metadata(root.join(path)).is_ok_and(|meta| meta.is_file())
    });
    let directories = directories(&files);
    Ok(Inventory { files, directories })
}
fn git_files(root: &Path) -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ])
        .current_dir(root)
        .output()?;
    anyhow::ensure!(output.status.success(), "cannot read Git inventory");
    output
        .stdout
        .split(|ch| *ch == 0)
        .filter(|name| !name.is_empty())
        .map(|name| Ok(PathBuf::from(String::from_utf8(name.to_vec())?)))
        .collect()
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
fn walk(root: &Path, relative: &Path, exclude: &GlobSet, files: &mut Vec<PathBuf>) -> Result<()> {
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
            walk(root, &path, exclude, files)?;
        } else if file {
            files.push(path);
        }
    }
    Ok(())
}
