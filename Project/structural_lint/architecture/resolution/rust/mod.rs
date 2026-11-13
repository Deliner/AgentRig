mod macro_origins;
mod module_tree;
#[cfg(test)]
mod tests;
mod use_aliases;

use super::Resolved;
use crate::structural_lint::architecture::extraction::{References, Target};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

struct Module {
    file: PathBuf,
    directory: PathBuf,
}

enum Located {
    Module(Vec<String>),
    Item(Vec<String>),
    External(String),
}

struct Query<'a> {
    source: &'a Path,
    visiting: BTreeSet<Vec<String>>,
}

// DECISION: D136
struct Pending {
    file: PathBuf,
    base: Vec<String>,
}

pub struct Rust<'a> {
    sources: &'a BTreeMap<PathBuf, References>,
    external: BTreeSet<String>,
    local: BTreeSet<String>,
    modules: BTreeMap<Vec<String>, Module>,
    files: BTreeMap<PathBuf, Vec<String>>,
    items: BTreeSet<Vec<String>>,
    imports: BTreeMap<Vec<String>, String>,
    // DECISION: D136
    included: BTreeMap<PathBuf, BTreeSet<Vec<String>>>,
    generated: BTreeSet<Vec<String>>,
    macros: BTreeMap<Vec<String>, PathBuf>,
    globs: BTreeMap<Vec<String>, Vec<String>>,
}

impl<'a> Rust<'a> {
    pub fn is_root(&self, path: &Path) -> bool {
        self.files.get(path).is_some_and(Vec::is_empty)
    }

    pub fn contains_source(&self, path: &Path) -> bool {
        self.files.contains_key(path) || self.included.contains_key(path)
    }

    pub fn new(
        root: &Path,
        sources: &'a BTreeMap<PathBuf, References>,
        external: &BTreeSet<String>,
        local: &BTreeMap<String, PathBuf>,
    ) -> Result<Self> {
        ensure!(
            sources.contains_key(root),
            "crate root is missing from source inventory: {}",
            root.display()
        );
        let mut resolver = Self::empty(sources);
        resolver.external = external.iter().chain(local.keys()).cloned().collect();
        resolver.local = local.keys().cloned().collect();
        resolver.load_tree(root)?;
        Ok(resolver)
    }

    fn empty(sources: &'a BTreeMap<PathBuf, References>) -> Self {
        Self {
            sources,
            external: BTreeSet::new(),
            local: BTreeSet::new(),
            modules: BTreeMap::new(),
            files: BTreeMap::new(),
            items: BTreeSet::new(),
            imports: BTreeMap::new(),
            included: BTreeMap::new(),
            generated: BTreeSet::new(),
            macros: BTreeMap::new(),
            globs: BTreeMap::new(),
        }
    }

    fn bases(&self, source: &Path) -> Vec<&Vec<String>> {
        self.files
            .get(source)
            .into_iter()
            .chain(self.included.get(source).into_iter().flatten())
            .collect()
    }

    // DECISION: D136
    pub fn resolve(&self, source: &Path, target: &Target) -> Result<Resolved> {
        let bases = self.bases(source);
        ensure!(
            !bases.is_empty(),
            "source is not in the declared crate module tree"
        );
        let mut combined = Resolved::default();
        for base in bases {
            let resolved = self.resolve_in(source, base, target)?;
            combined.files.extend(resolved.files);
            combined.external = combined.external.or(resolved.external);
        }
        Ok(combined)
    }

    fn resolve_in(&self, source: &Path, base: &[String], target: &Target) -> Result<Resolved> {
        match target {
            Target::RustMacro(call) => self.macro_result(source, base, call),
            Target::RustDerive { path, scope } => self.derive_result(path, &[base, scope].concat()),
            Target::RustModule { path, .. } => {
                let key = [base, path].concat();
                let module = self
                    .modules
                    .get(&key)
                    .context("module declaration was not resolved")?;
                Ok(Resolved {
                    files: BTreeSet::from([module.file.clone()]),
                    external: None,
                })
            }
            Target::RustPath { path, scope } => self.path_result(path, &[base, scope].concat()),
            Target::RustInclude { file, .. } => self.include_result(source, file),
            Target::PythonModule(_) | Target::PythonFrom { .. } => {
                bail!("not a Rust module reference")
            }
        }
    }
}
