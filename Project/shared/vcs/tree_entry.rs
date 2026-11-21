use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FileKind {
    File,
    Executable,
    Symlink,
    Submodule,
}

impl FileKind {
    pub fn mode(self) -> &'static str {
        match self {
            Self::File => "100644",
            Self::Executable => "100755",
            Self::Symlink => "120000",
            Self::Submodule => "160000",
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub kind: FileKind,
    pub object: String,
}
