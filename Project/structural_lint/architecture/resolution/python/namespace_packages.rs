use super::{Python, Resolved, module_parts};
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub(super) struct Module {
    files: BTreeSet<PathBuf>,
    package: Option<PathBuf>,
}

impl Python<'_> {
    pub(super) fn local(&self, parts: &[&str], names: &[String]) -> Result<Resolved> {
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

    pub(super) fn entry(&self, parent: &Path, name: &str) -> Option<Module> {
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
