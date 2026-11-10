// DECISION: D106
#[cfg(test)]
mod tests;

use std::{
    fmt,
    fs::{DirEntry, Metadata},
    io,
    path::{Path, PathBuf},
};

pub use std::fs::{File, OpenOptions, Permissions};
pub use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

pub type Result<T> = std::result::Result<T, Error>;

// DECISION: D106
#[derive(Debug)]
pub struct Error {
    action: &'static str,
    path: PathBuf,
    source: io::Error,
}

impl Error {
    pub fn kind(&self) -> io::ErrorKind {
        self.source.kind()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{action} {path}",
            action = self.action,
            path = self.path.display()
        )
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

fn at<T>(action: &'static str, path: &Path, result: io::Result<T>) -> Result<T> {
    result.map_err(|source| Error {
        action,
        path: path.into(),
        source,
    })
}

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

pub fn symlink(target: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    at(
        "link",
        link.as_ref(),
        std::os::unix::fs::symlink(&target, &link),
    )
}

pub fn remove_file(path: impl AsRef<Path>) -> Result<()> {
    at("remove", path.as_ref(), std::fs::remove_file(&path))
}

pub fn remove_dir_all(path: impl AsRef<Path>) -> Result<()> {
    // DECISION: D106
    at("remove", path.as_ref(), std::fs::remove_dir_all(&path))
}
