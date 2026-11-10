use super::{Module, Pending, Rust, joined};
use crate::structural_lint::architecture::resolution::{Resolved, normalize};
use crate::structural_lint::architecture::source::{Reference, References, Target};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

impl<'a> Rust<'a> {
    // DECISION: D144
    pub fn modules(
        root: &Path,
        sources: &'a BTreeMap<PathBuf, References>,
    ) -> BTreeMap<PathBuf, Vec<Vec<String>>> {
        let mut tree = Self::empty(sources);
        let planted = sources.contains_key(root) && tree.plant(root).is_ok();
        let mut pending: Vec<_> = planted.then(|| Pending::root(root)).into_iter().collect();
        while let Some(next) = pending.pop() {
            for reference in &sources[&next.file].items {
                pending.extend(tree.declare(&next.base, reference).ok().flatten());
                pending.extend(
                    tree.include(&next.file, &next.base, reference)
                        .ok()
                        .flatten(),
                );
            }
        }
        let files = tree.files.keys().chain(tree.included.keys());
        files
            .map(|file| {
                (
                    file.clone(),
                    tree.bases(file).into_iter().cloned().collect(),
                )
            })
            .collect()
    }
}

impl Pending {
    fn root(root: &Path) -> Self {
        Self {
            file: root.to_path_buf(),
            base: Vec::new(),
        }
    }
}

impl Rust<'_> {
    // DECISION: D136
    pub(super) fn load_tree(&mut self, root: &Path) -> Result<()> {
        self.plant(root)?;
        let mut pending = vec![Pending::root(root)];
        while let Some(next) = pending.pop() {
            pending.extend(self.load(&next.file, &next.base)?);
        }
        self.validate_expansions()
    }

    fn plant(&mut self, root: &Path) -> Result<()> {
        self.modules.insert(
            Vec::new(),
            Module {
                file: root.into(),
                directory: root.parent().context("crate root has no parent")?.into(),
            },
        );
        self.files.insert(root.into(), Vec::new());
        Ok(())
    }

    fn load(&mut self, file: &Path, base: &[String]) -> Result<Vec<Pending>> {
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
        let mut pending = Vec::new();
        for reference in &source.items {
            pending.extend(self.declare(base, reference)?);
            pending.extend(self.include(file, base, reference)?);
        }
        for item in &source.rust_items {
            let mut key = joined(base, &item.scope);
            key.push(item.name.clone());
            self.items.insert(key);
        }
        self.expansions(file, base);
        for import in &source.rust_imports {
            let mut key = joined(base, &import.scope);
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

    // DECISION: D136
    fn expansions(&mut self, file: &Path, base: &[String]) {
        let source = &self.sources[file];
        for item in &source.rust_macros {
            let mut key = joined(base, &item.scope);
            key.push(item.name.clone());
            self.macros.insert(key, file.into());
        }
        for scope in &source.rust_generated {
            self.generated.insert(joined(base, scope));
        }
        for import in source
            .rust_imports
            .iter()
            .filter(|import| import.name == "*")
        {
            let globs = self.globs.entry(joined(base, &import.scope)).or_default();
            globs.push(import.path.clone());
        }
    }

    // DECISION: D136
    fn include(
        &mut self,
        file: &Path,
        base: &[String],
        reference: &Reference,
    ) -> Result<Option<Pending>> {
        let Target::RustInclude {
            file: included,
            scope,
        } = &reference.target
        else {
            return Ok(None);
        };
        let target = included_file(file, included)?;
        let base = joined(base, scope);
        let fresh = self.sources.contains_key(&target)
            && self
                .included
                .entry(target.clone())
                .or_default()
                .insert(base.clone());
        Ok(fresh.then_some(Pending { file: target, base }))
    }

    pub(super) fn include_result(&self, source: &Path, file: &str) -> Result<Resolved> {
        let target = included_file(source, file)?;
        ensure!(
            self.sources.contains_key(&target),
            "included Rust source is absent from source inventory: {}",
            target.display()
        );
        Ok(Resolved {
            files: BTreeSet::from([target]),
            external: None,
        })
    }

    fn declare(&mut self, base: &[String], reference: &Reference) -> Result<Option<Pending>> {
        let Target::RustModule { path, inline, .. } = &reference.target else {
            return Ok(None);
        };
        let (_, parent) = path.split_last().context("empty Rust module declaration")?;
        let key = joined(base, path);
        let parent = self
            .modules
            .get(&joined(base, parent))
            .context("parent module is unresolved")?;
        let module = self.module_location(parent, reference)?;
        let pending = (!inline).then(|| Pending {
            file: module.file.clone(),
            base: key.clone(),
        });
        let duplicate = self.modules.insert(key.clone(), module).is_some();
        ensure!(!duplicate, "duplicate Rust module {}", key.join("::"));
        if let Some(next) = &pending {
            ensure!(
                !self.files.contains_key(&next.file),
                "Rust source is declared by multiple modules: {}",
                next.file.display()
            );
            self.files.insert(next.file.clone(), key);
        }
        Ok(pending)
    }

    fn module_location(&self, parent: &Module, reference: &Reference) -> Result<Module> {
        let Target::RustModule { path, inline, file } = &reference.target else {
            bail!("expected Rust module declaration");
        };
        if let Some(value) = file {
            return self.explicit_module(parent, value);
        }
        let name = path.last().context("empty Rust module declaration")?;
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
        Ok(Module { file, directory })
    }

    fn explicit_module(&self, parent: &Module, value: &str) -> Result<Module> {
        let file = normalize(
            &parent
                .file
                .parent()
                .context("module source parent required")?
                .join(value),
        )?;
        ensure!(
            self.sources.contains_key(&file),
            "explicit Rust module source is missing: {}",
            file.display()
        );
        let directory = file
            .parent()
            .context("module source parent required")?
            .into();
        Ok(Module { file, directory })
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

fn included_file(source: &Path, file: &str) -> Result<PathBuf> {
    let parent = source.parent().context("included source has no parent")?;
    normalize(&parent.join(file))
}
