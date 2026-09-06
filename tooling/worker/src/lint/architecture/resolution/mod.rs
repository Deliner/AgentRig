//! Resolve references against the source inventory without executing project code.
pub mod javascript;
pub mod python;

use std::{collections::BTreeSet, path::PathBuf};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Resolved {
    pub files: BTreeSet<PathBuf>,
    pub external: Option<String>,
}
