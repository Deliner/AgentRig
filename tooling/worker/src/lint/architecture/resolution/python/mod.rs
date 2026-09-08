//! Static Python resolution under one explicit import root. Runtime import hooks,
//! path mutations and native/zip modules are outside this resolver's coverage.
//! Targets are .py files; a .pyi stub is not treated as a runtime module.
//! Namespace-only access and ambiguous package members return errors until their
//! directory boundaries and initializer exports can be represented explicitly.
#[cfg(test)]
mod tests;

use super::Resolved;
use crate::lint::architecture::source::Target;
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

pub struct Python<'a> {
    root: PathBuf,
    files: &'a BTreeSet<PathBuf>,
    external: &'a BTreeSet<String>,
}

struct Module {
    files: BTreeSet<PathBuf>,
    package: Option<PathBuf>,
}

impl<'a> Python<'a> {
    /// Files come from the normalized project inventory, including files outside lint scope.
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
            _ => bail!("not a Python module reference"),
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

    fn local(&self, parts: &[&str], names: &[String]) -> Result<Resolved> {
        let mut loaded = self.module(parts)?;
        let namespace_only = names.is_empty()
            && loaded
                .package
                .as_ref()
                .is_some_and(|path| self.source_file(&path.join("__init__")).is_none());
        ensure!(
            !namespace_only,
            "namespace-only import has no source file boundary: {}",
            parts.join(".")
        );
        self.members(&mut loaded, names)?;
        ensure!(
            !loaded.files.is_empty(),
            "namespace-only import has no source file boundary: {}",
            parts.join(".")
        );
        Ok(Resolved {
            files: loaded.files,
            external: None,
        })
    }

    fn absolute(&self, source: &Path, module: &str) -> Result<String> {
        let depth = module.bytes().take_while(|byte| *byte == b'.').count();
        let absolute = depth == 0;
        if absolute {
            return Ok(module.into());
        }
        let parent = source.parent().context("Python source has no parent")?;
        let package = parent.strip_prefix(&self.root)?;
        let mut parts: Vec<_> = package.iter().map(|part| part.to_string_lossy()).collect();
        ensure!(
            depth <= parts.len(),
            "relative import escapes its package: {module}"
        );
        parts.truncate(parts.len() + 1 - depth);
        let suffix = &module[depth..];
        let has_suffix = !suffix.is_empty();
        if has_suffix {
            parts.extend(module_parts(suffix)?.into_iter().map(Into::into));
        }
        Ok(parts.join("."))
    }

    fn module(&self, parts: &[&str]) -> Result<Module> {
        let mut loaded = Module {
            files: BTreeSet::new(),
            package: Some(self.root.clone()),
        };
        for part in parts {
            let parent = loaded
                .package
                .as_ref()
                .context("import traverses a non-package module")?;
            let next = self.entry(parent, part).with_context(|| {
                format!(
                    "unresolved Python module {}; configure its import root or external module",
                    parts.join(".")
                )
            })?;
            loaded.files.extend(next.files);
            loaded.package = next.package;
        }
        Ok(loaded)
    }

    fn entry(&self, parent: &Path, name: &str) -> Option<Module> {
        let path = parent.join(name);
        if let Some(initializer) = self.source_file(&path.join("__init__")) {
            return Some(Module {
                files: BTreeSet::from([initializer]),
                package: Some(path),
            });
        }
        if let Some(file) = self.source_file(&path) {
            return Some(Module {
                files: BTreeSet::from([file]),
                package: None,
            });
        }
        let namespace = self.files.iter().any(|file| file.starts_with(&path));
        namespace.then_some(Module {
            files: BTreeSet::new(),
            package: Some(path),
        })
    }

    fn source_file(&self, stem: &Path) -> Option<PathBuf> {
        let path = stem.with_extension("py");
        self.files.contains(&path).then_some(path)
    }

    fn members(&self, loaded: &mut Module, names: &[String]) -> Result<()> {
        let Some(package) = &loaded.package else {
            return Ok(());
        };
        let regular_package = self.source_file(&package.join("__init__")).is_some();
        for name in names {
            let wildcard = name == "*";
            ensure!(
                !wildcard,
                "package wildcard import requires resolving __all__"
            );
            module_parts(name)?;
            if let Some(member) = self.entry(package, name) {
                ensure!(
                    !regular_package,
                    "package member {name} may be an attribute or submodule; initializer exports require analysis"
                );
                ensure!(
                    !member.files.is_empty(),
                    "namespace member {name} has no source file boundary"
                );
                loaded.files.extend(member.files);
            } else {
                ensure!(regular_package, "unresolved namespace member {name}");
            }
        }
        Ok(())
    }
}

fn relative(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(name) => normalized.push(name),
            Component::CurDir => {}
            _ => bail!(
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
