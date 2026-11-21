use std::{collections::BTreeSet, path::PathBuf};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Resolved {
    pub files: BTreeSet<PathBuf>,
    // DECISION: D227
    pub initializers: BTreeSet<PathBuf>,
    pub external: Option<String>,
}
