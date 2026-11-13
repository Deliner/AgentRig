mod namespace_packages;
mod relative_imports;
#[cfg(test)]
mod tests;

use super::Resolved;
use crate::structural_lint::architecture::extraction::Target;
use anyhow::{Result, bail, ensure};
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

pub struct Python<'a> {
    root: PathBuf,
    files: &'a BTreeSet<PathBuf>,
    external: &'a BTreeSet<String>,
}

impl<'a> Python<'a> {
    pub fn new(
        root: &Path,
        files: &'a BTreeSet<PathBuf>,
        external: &'a BTreeSet<String>,
    ) -> Result<Self> {
        let root = relative(root)?;
        for name in external {
            module_parts(name)?;
        }
        Ok(Self {
            root,
            files,
            external,
        })
    }

    pub fn resolve(&self, source: &Path, target: &Target) -> Result<Resolved> {
        ensure!(
            self.files.contains(source),
            "source is missing from inventory"
        );
        ensure!(
            source.starts_with(&self.root),
            "source is outside Python import root"
        );
        let (module, names) = match target {
            Target::PythonModule(module) => (module, &[][..]),
            Target::PythonFrom { module, names } => (module, names.as_slice()),
            Target::RustPath { .. }
            | Target::RustMacro(_)
            | Target::RustDerive { .. }
            | Target::RustModule { .. }
            | Target::RustInclude { .. } => bail!("not a Python module reference"),
        };
        let absolute = self.absolute(source, module)?;
        let parts = module_parts(&absolute)?;
        let local = self.entry(&self.root, parts[0]).is_some();
        let external = self
            .external
            .iter()
            .any(|name| absolute == *name || absolute.starts_with(&format!("{name}.")));
        let external_only = !local && external && !module.starts_with('.');
        if external_only {
            return Ok(Resolved {
                files: BTreeSet::new(),
                external: Some(absolute),
            });
        }
        self.local(&parts, names)
    }
}

fn relative(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(name) => normalized.push(name),
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => bail!(
                "Python resolution paths must stay project-relative: {}",
                path.display()
            ),
        }
    }
    Ok(normalized)
}

fn module_parts(module: &str) -> Result<Vec<&str>> {
    let parts: Vec<_> = module.split('.').collect();
    let valid = parts.iter().all(|part| {
        let mut chars = part.chars();
        chars
            .next()
            .is_some_and(|ch| ch == '_' || ch.is_alphabetic())
            && chars.all(|ch| ch == '_' || ch.is_alphanumeric())
    });
    ensure!(valid, "unsupported Python module name: {module}");
    Ok(parts)
}
