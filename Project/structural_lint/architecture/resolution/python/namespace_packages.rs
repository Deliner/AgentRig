use super::resolver::{Python, module_parts};
use crate::structural_lint::architecture::resolution::resolved::Resolved;
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub(super) struct Module {
    files: BTreeSet<PathBuf>,
    package: Option<PathBuf>,
}

// DECISION: D227
struct Loaded {
    module: Module,
    initializers: BTreeSet<PathBuf>,
}

impl Python<'_> {
    pub(super) fn local(&self, parts: &[&str], names: &[String]) -> Result<Resolved> {
        let Loaded {
            module: mut loaded,
            initializers,
        } = self.module(parts)?;
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
            initializers,
            external: None,
        })
    }

    // DECISION: D227
    fn module(&self, parts: &[&str]) -> Result<Loaded> {
        let mut loaded = Loaded {
            module: Module {
                files: BTreeSet::new(),
                package: Some(self.root.clone()),
            },
            initializers: BTreeSet::new(),
        };
        for part in parts {
            let parent = loaded
                .module
                .package
                .as_ref()
                .context("import traverses a non-package module")?;
            let next = self.entry(parent, part).with_context(|| {
                format!(
                    "unresolved Python module {}; configure its import root or external module",
                    parts.join(".")
                )
            })?;
            let passed = std::mem::replace(&mut loaded.module, next);
            loaded.initializers.extend(passed.files);
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
