use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq)]
pub enum Entry {
    Directory,
    File(Vec<u8>),
}

pub type Snapshot = BTreeMap<PathBuf, Entry>;

fn entries(directory: &Path, found: &mut Snapshot) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let nested = path.is_dir();
        if nested {
            entries(&path, found);
            found.insert(path, Entry::Directory);
        } else {
            let contents = fs::read(&path).unwrap();
            found.insert(path, Entry::File(contents));
        }
    }
}

pub fn snapshot(root: &Path) -> Snapshot {
    let mut found = BTreeMap::new();
    entries(root, &mut found);
    found
}
