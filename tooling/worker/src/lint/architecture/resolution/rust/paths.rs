use super::{Located, Query, Rust};
use crate::lint::architecture::resolution::Resolved;
use anyhow::{Context, Result, bail, ensure};
use std::{collections::BTreeSet, path::Path};

impl Rust<'_> {
    pub(super) fn path_result(
        &self,
        source: &Path,
        path: &str,
        scope: &[String],
    ) -> Result<Resolved> {
        ensure!(
            self.modules.contains_key(scope),
            "Rust module scope is unresolved"
        );
        let mut query = Query {
            source,
            visiting: BTreeSet::new(),
        };
        match self.path(scope, path, &mut query)? {
            Located::Module(module) | Located::Item(module) => self.module_result(&module),
            Located::External(path) => Ok(Resolved {
                files: BTreeSet::new(),
                external: Some(path),
            }),
        }
    }

    pub(super) fn path(
        &self,
        scope: &[String],
        path: &str,
        query: &mut Query<'_>,
    ) -> Result<Located> {
        let parts: Vec<_> = path.trim_start_matches("::").split("::").collect();
        ensure!(
            parts.iter().all(|part| !part.is_empty()),
            "empty Rust path component"
        );
        let absolute_external = path.starts_with("::");
        if absolute_external {
            return self.external_path(path, parts[0]);
        }
        let external =
            !matches!(parts[0], "crate" | "self" | "super") && self.external_name(scope, parts[0]);
        if external {
            return self.external_path(path, parts[0]);
        }
        let (mut module, start) = match parts[0] {
            "crate" => (Vec::new(), 1),
            "self" => (scope.to_vec(), 1),
            _ => (scope.to_vec(), 0),
        };
        let mut suffix = &parts[start..];
        while suffix.first() == Some(&"super") {
            ensure!(module.pop().is_some(), "super escapes the crate root");
            suffix = &suffix[1..];
        }
        self.walk(module, suffix, query)
    }

    fn walk(
        &self,
        mut module: Vec<String>,
        parts: &[&str],
        query: &mut Query<'_>,
    ) -> Result<Located> {
        for (index, part) in parts.iter().enumerate() {
            let mut key = module.clone();
            key.push((*part).into());
            let child_module = self.modules.contains_key(&key);
            if child_module {
                module = key;
                continue;
            }
            let imported = self.imports.contains_key(&key);
            if imported {
                return self.import(&key, &parts[index + 1..], query);
            }
            let item = self.items.contains(&key);
            if item {
                return Ok(Located::Item(module));
            }
            bail!(
                "unresolved Rust name {}; imported, generated or lexical bindings require analysis",
                key.join("::")
            );
        }
        Ok(Located::Module(module))
    }

    fn import(&self, key: &[String], suffix: &[&str], query: &mut Query<'_>) -> Result<Located> {
        ensure!(
            query.visiting.insert(key.into()),
            "cyclic Rust import alias {}",
            key.join("::")
        );
        let owner = &key[..key.len() - 1];
        let imported = self.path(owner, &self.imports[key], query);
        query.visiting.remove(key);
        match imported? {
            Located::Module(module) => self.walk(module, suffix, query),
            Located::Item(module) => {
                let through_facade = self.modules[owner].file != query.source;
                let boundary = if through_facade {
                    owner.to_vec()
                } else {
                    module
                };
                Ok(Located::Item(boundary))
            }
            Located::External(path) => {
                let through_facade = self.modules[owner].file != query.source;
                if through_facade {
                    return Ok(Located::Item(owner.to_vec()));
                }
                let path = match suffix.is_empty() {
                    true => path,
                    false => format!("{path}::{}", suffix.join("::")),
                };
                Ok(Located::External(path))
            }
        }
    }

    pub(super) fn module_result(&self, module: &[String]) -> Result<Resolved> {
        let module = self
            .modules
            .get(module)
            .context("Rust module is unresolved")?;
        Ok(Resolved {
            files: BTreeSet::from([module.file.clone()]),
            external: None,
        })
    }

    fn external_name(&self, scope: &[String], name: &str) -> bool {
        let mut key = scope.to_vec();
        key.push(name.into());
        self.external.contains(name)
            && !self.modules.contains_key(&key)
            && !self.items.contains(&key)
            && !self.imports.contains_key(&key)
    }

    fn external_path(&self, path: &str, first: &str) -> Result<Located> {
        ensure!(
            self.external.contains(first),
            "undeclared external Rust crate {first}"
        );
        Ok(Located::External(path.into()))
    }
}
