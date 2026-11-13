// DECISION: D106
mod path_errors;
#[cfg(test)]
mod tests;

use path_errors::at;
use std::{
    ffi::OsStr,
    fs::{DirEntry, Metadata},
    path::{Path, PathBuf},
    time::SystemTime,
};

pub use path_errors::{Error, Result, present};
pub use std::fs::{File, OpenOptions, Permissions};
pub use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

pub fn read(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    at("read", path.as_ref(), std::fs::read(&path))
}

pub fn read_to_string(path: impl AsRef<Path>) -> Result<String> {
    at("read", path.as_ref(), std::fs::read_to_string(&path))
}

pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
    at("write", path.as_ref(), std::fs::write(&path, contents))
}

pub fn open(path: impl AsRef<Path>) -> Result<File> {
    at("open", path.as_ref(), File::open(&path))
}

pub fn open_with(options: &OpenOptions, path: impl AsRef<Path>) -> Result<File> {
    at("open", path.as_ref(), options.open(&path))
}

pub fn lock(path: impl AsRef<Path>) -> Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(false).write(true);
    let file = open_with(&options, &path)?;
    at("lock", path.as_ref(), file.lock())?;
    Ok(file)
}

pub fn set_modified(path: impl AsRef<Path>, time: SystemTime) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true);
    let file = open_with(&options, &path)?;
    at(
        "set the modification time of",
        path.as_ref(),
        file.set_modified(time),
    )
}

pub struct Move<'a> {
    pub from: &'a Path,
    pub to: &'a Path,
}

pub fn rename(paths: Move<'_>) -> Result<()> {
    at("move", paths.from, std::fs::rename(paths.from, paths.to))
}

pub fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    at(
        "create directory",
        path.as_ref(),
        std::fs::create_dir_all(&path),
    )
}

pub fn read_dir(path: impl AsRef<Path>) -> Result<impl Iterator<Item = Result<DirEntry>>> {
    let directory = path.as_ref().to_path_buf();
    let entries = at("read directory", &directory, std::fs::read_dir(&directory))?;
    Ok(entries.map(move |entry| at("read directory", &directory, entry)))
}

pub fn metadata(path: impl AsRef<Path>) -> Result<Metadata> {
    at("inspect", path.as_ref(), std::fs::metadata(&path))
}

// DECISION: D125
pub fn file_metadata(file: &File, path: impl AsRef<Path>) -> Result<Metadata> {
    at("inspect", path.as_ref(), file.metadata())
}

pub fn try_exists(path: impl AsRef<Path>) -> Result<bool> {
    at("inspect", path.as_ref(), path.as_ref().try_exists())
}

pub fn symlink_metadata(path: impl AsRef<Path>) -> Result<Metadata> {
    at("inspect", path.as_ref(), std::fs::symlink_metadata(&path))
}

pub fn read_link(path: impl AsRef<Path>) -> Result<PathBuf> {
    at("read link", path.as_ref(), std::fs::read_link(&path))
}

pub fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf> {
    at("resolve", path.as_ref(), std::fs::canonicalize(&path))
}

pub fn set_permissions(path: impl AsRef<Path>, permissions: Permissions) -> Result<()> {
    at(
        "set permissions on",
        path.as_ref(),
        std::fs::set_permissions(&path, permissions),
    )
}

pub fn symlink(target: &OsStr, link: impl AsRef<Path>) -> Result<()> {
    at(
        "link",
        link.as_ref(),
        std::os::unix::fs::symlink(target, &link),
    )
}

pub fn remove_file(path: impl AsRef<Path>) -> Result<()> {
    at("remove", path.as_ref(), std::fs::remove_file(&path))
}

pub fn remove_dir_all(path: impl AsRef<Path>) -> Result<()> {
    // DECISION: D106
    at("remove", path.as_ref(), std::fs::remove_dir_all(&path))
}
