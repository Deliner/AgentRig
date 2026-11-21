mod contract_check;
mod contract_rules;
pub mod extraction;
pub mod resolution;

pub use contract_check::{
    Edges, contains_directory, edges, entry_bypasses, entry_uses, file_cycles, run, validate,
};
pub use contract_rules::Rooted;
// DECISION: D229
pub use extraction::Target;
pub use resolution::python::Python;

use anyhow::Result;
use resolution::rust::Rust;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// DECISION: D144
pub fn crate_modules(
    texts: &BTreeMap<PathBuf, String>,
    crate_roots: &[String],
) -> Result<BTreeMap<PathBuf, Vec<Vec<String>>>> {
    let mut sources = BTreeMap::new();
    for (path, text) in texts {
        sources.insert(path.clone(), extraction::extract(path, text)?);
    }
    let mut modules: BTreeMap<PathBuf, Vec<Vec<String>>> = BTreeMap::new();
    for crate_root in crate_roots {
        for (file, bases) in Rust::modules(Path::new(crate_root), &sources) {
            modules.entry(file).or_default().extend(bases);
        }
    }
    Ok(modules)
}
