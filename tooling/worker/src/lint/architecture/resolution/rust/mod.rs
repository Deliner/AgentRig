//! Declared Rust module trees and source paths for Rust 2018 or later.
//! Compiler expansion and block-local imported-name lookup remain explicit limits.
mod paths;
#[cfg(test)]
mod tests;
mod tree;

use super::Resolved;
use crate::lint::architecture::source::{Reference, References, Target};
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

pub struct Rust<'a> {
    sources: &'a BTreeMap<PathBuf, References>,
    external: &'a BTreeSet<String>,
    modules: BTreeMap<Vec<String>, Module>,
    files: BTreeMap<PathBuf, Vec<String>>,
    items: BTreeSet<Vec<String>>,
    imports: BTreeMap<Vec<String>, String>,
}

impl<'a> Rust<'a> {
    pub fn new(
        root: &Path,
        sources: &'a BTreeMap<PathBuf, References>,
        external: &'a BTreeSet<String>,
    ) -> Result<Self> {
        ensure!(
            sources.contains_key(root),
            "crate root is missing from source inventory: {}",
            root.display()
        );
        let mut resolver = Self {
            sources,
            external,
            modules: BTreeMap::new(),
            files: BTreeMap::new(),
            items: BTreeSet::new(),
            imports: BTreeMap::new(),
        };
        resolver.modules.insert(
            Vec::new(),
            Module {
                file: root.into(),
                directory: root.parent().context("crate root has no parent")?.into(),
            },
        );
        resolver.files.insert(root.into(), Vec::new());
        let mut pending = vec![root.to_path_buf()];
        while let Some(file) = pending.pop() {
            pending.extend(resolver.load(&file)?);
        }
        Ok(resolver)
    }

    pub fn resolve(&self, source: &Path, target: &Target) -> Result<Resolved> {
        let base = self
            .files
            .get(source)
            .context("source is not in the declared crate module tree")?;
        match target {
            Target::RustModule { path, .. } => {
                let key = joined(base, path);
                let module = self
                    .modules
                    .get(&key)
                    .context("module declaration was not resolved")?;
                Ok(Resolved {
                    files: BTreeSet::from([module.file.clone()]),
                    external: None,
                })
            }
            Target::RustPath { path, scope } => {
                let scope = joined(base, scope);
                ensure!(
                    self.modules.contains_key(&scope),
                    "Rust module scope is unresolved"
                );
                let mut query = Query {
                    source,
                    visiting: BTreeSet::new(),
                };
                match self.path(&scope, path, &mut query)? {
                    Located::Module(module) | Located::Item(module) => self.module_result(&module),
                    Located::External(path) => Ok(Resolved {
                        files: BTreeSet::new(),
                        external: Some(path),
                    }),
                }
            }
            _ => bail!("not a Rust module reference"),
        }
    }
}

fn joined(base: &[String], suffix: &[String]) -> Vec<String> {
    base.iter().chain(suffix).cloned().collect()
}
