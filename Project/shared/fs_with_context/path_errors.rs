use std::{
    fmt, io,
    path::{Path, PathBuf},
};

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
    pub fn missing(&self) -> bool {
        // DECISION: D156
        self.source.kind() == io::ErrorKind::NotFound
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

pub fn present<T>(result: Result<T>) -> Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) => {
            let kind = error.kind();
            // DECISION: D156
            let absent = matches!(kind, io::ErrorKind::NotFound | io::ErrorKind::NotADirectory);
            if absent { Ok(None) } else { Err(error) }
        }
    }
}

pub(super) fn at<T>(action: &'static str, path: &Path, result: io::Result<T>) -> Result<T> {
    result.map_err(|source| Error {
        action,
        path: path.into(),
        source,
    })
}
