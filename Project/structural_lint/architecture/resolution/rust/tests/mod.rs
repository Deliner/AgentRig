mod aliases;
mod compiled;
mod expansion;
mod external;
mod module_tree;

use crate::structural_lint::architecture::extraction::{self, References, Target};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

fn sources(files: &[(&str, &str)]) -> BTreeMap<PathBuf, References> {
    files
        .iter()
        .map(|(path, text)| {
            (
                PathBuf::from(path),
                extraction::extract(Path::new(path), text).unwrap(),
            )
        })
        .collect()
}

fn path(value: &str) -> Target {
    Target::RustPath {
        path: value.into(),
        scope: Vec::new(),
    }
}

fn expected(file: &str) -> BTreeSet<PathBuf> {
    BTreeSet::from([file.into()])
}
