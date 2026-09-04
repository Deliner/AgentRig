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
    let mut files = Vec::new();
    if root.join(".git").exists() {
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
        for name in output
            .stdout
            .split(|ch| *ch == 0)
            .filter(|name| !name.is_empty())
        {
            files.push(PathBuf::from(String::from_utf8(name.to_vec())?));
        }
    } else {
        walk(root, Path::new(""), exclude, &mut files)?;
    }
    files.sort();
    files.dedup();
    files.retain(|path| {
        !exclude.is_match(path)
            && fs::symlink_metadata(root.join(path)).is_ok_and(|meta| meta.is_file())
    });
    let mut directories: BTreeMap<PathBuf, BTreeSet<String>> = BTreeMap::new();
    for path in &files {
        let mut child = path.as_path();
        while let Some(parent) = child.parent() {
            directories
                .entry(if parent.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    parent.into()
                })
                .or_default()
                .insert(child.file_name().unwrap().to_string_lossy().into_owned());
            child = parent;
        }
    }
    Ok(Inventory { files, directories })
}
fn walk(root: &Path, relative: &Path, exclude: &GlobSet, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root.join(relative))? {
        let entry = entry?;
        let path = relative.join(entry.file_name());
        if exclude.is_match(&path) || entry.file_type()?.is_symlink() {
            continue;
        }
        if entry.file_type()?.is_dir() {
            walk(root, &path, exclude, files)?;
        } else if entry.file_type()?.is_file() {
            files.push(path);
        }
    }
    Ok(())
}
