use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FileKind {
    File,
    Executable,
    Symlink,
    Submodule,
}

#[derive(Debug)]
pub struct Entry {
    pub kind: FileKind,
    pub object: String,
}
