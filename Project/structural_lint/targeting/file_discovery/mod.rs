mod unversioned;
mod versioned;

pub use unversioned::unversioned;
pub use versioned::collect;

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

// DECISION: D016
pub struct Inventory {
    pub files: Vec<PathBuf>,
    pub directories: BTreeMap<PathBuf, BTreeSet<String>>,
}
impl Inventory {
    pub fn file_entries(&self, directory: &Path, uncounted: &str) -> usize {
        let root = directory == Path::new(".");
        let prefix = if root { Path::new("") } else { directory };
        self.files
            .iter()
            .filter_map(|file| file.strip_prefix(prefix).ok()?.components().next())
            .filter(|name| name.as_os_str() != uncounted)
            .collect::<BTreeSet<_>>()
            .len()
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
            if let Some(name) = child.file_name() {
                directories
                    .entry(directory)
                    .or_default()
                    .insert(name.to_string_lossy().into_owned());
            }
            child = parent;
        }
    }
    directories
}
