use super::{Module, Rust, joined};
use crate::lint::architecture::source::{Reference, Target};
use anyhow::{Context, Result, bail, ensure};
use std::path::{Path, PathBuf};

impl Rust<'_> {
    pub(super) fn load(&mut self, file: &Path) -> Result<Vec<PathBuf>> {
        let sources = self.sources;
        let source = sources
            .get(file)
            .context("declared module source is missing")?;
        if let Some(issue) = source.issues.first() {
            bail!(
                "{}:{}: {}",
                file.display(),
                issue.line.unwrap_or(1),
                issue.message
            );
        }
        let base = self.files[file].clone();
        let mut pending = Vec::new();
        for reference in &source.items {
            if let Some(file) = self.declare(&base, reference)? {
                pending.push(file);
            }
        }
        for item in &source.rust_items {
            let mut key = joined(&base, &item.scope);
            key.push(item.name.clone());
            self.items.insert(key);
        }
        for import in &source.rust_imports {
            let mut key = joined(&base, &import.scope);
            key.push(import.name.clone());
            let ordinary = !matches!(import.name.as_str(), "_" | "*");
            if ordinary {
                ensure!(
                    self.imports.insert(key, import.path.clone()).is_none(),
                    "ambiguous Rust import binding {} in {}",
                    import.name,
                    file.display()
                );
            }
        }
        Ok(pending)
    }

    fn declare(&mut self, base: &[String], reference: &Reference) -> Result<Option<PathBuf>> {
        let Target::RustModule { path, inline } = &reference.target else {
            return Ok(None);
        };
        let (name, parent) = path.split_last().context("empty Rust module declaration")?;
        let key = joined(base, path);
        let parent = self
            .modules
            .get(&joined(base, parent))
            .context("parent module is unresolved")?;
        let directory = parent.directory.join(name);
        let file = match inline {
            true => parent.file.clone(),
            false => self.module_file(&directory).with_context(|| {
                format!(
                    "{}:{}: module {name}",
                    parent.file.display(),
                    reference.line
                )
            })?,
        };
        let pending = (!inline).then(|| file.clone());
        let duplicate = self
            .modules
            .insert(
                key.clone(),
                Module {
                    file: file.clone(),
                    directory,
                },
            )
            .is_some();
        ensure!(!duplicate, "duplicate Rust module {}", key.join("::"));
        if let Some(file) = &pending {
            self.files.insert(file.clone(), key);
        }
        Ok(pending)
    }

    fn module_file(&self, directory: &Path) -> Result<PathBuf> {
        let candidates: Vec<_> = [directory.with_extension("rs"), directory.join("mod.rs")]
            .into_iter()
            .filter(|file| self.sources.contains_key(file))
            .collect();
        ensure!(
            candidates.len() == 1,
            "expected exactly one module source (name.rs or name/mod.rs), found {}",
            candidates.len()
        );
        Ok(candidates[0].clone())
    }
}
