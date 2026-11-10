use super::{Located, Query, Rust};
use crate::structural_lint::architecture::resolution::Resolved;
use crate::structural_lint::architecture::source::prelude_origin;
use anyhow::{Context, Result, bail, ensure};
use std::collections::BTreeSet;

impl Rust<'_> {
    pub(super) fn path_result(&self, path: &str, scope: &[String]) -> Result<Resolved> {
        let module = self
            .modules
            .get(scope)
            .context("Rust module scope is unresolved")?;
        let mut query = Query {
            source: &module.file,
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
            // DECISION: D136
            return self.unbound(module, &parts[index..], query);
        }
        Ok(Located::Module(module))
    }

    // DECISION: D136
    fn unbound(
        &self,
        module: Vec<String>,
        parts: &[&str],
        query: &mut Query<'_>,
    ) -> Result<Located> {
        let name = parts[0];
        let globs = self.glob_sources(&module, query)?;
        if let Some(target) = glob_module(&globs, |target| self.binds(target, name)) {
            let found = self.walk(target, &parts[..1], query)?;
            return self.reexport(&module, found, &parts[1..], query);
        }
        let prelude = (!globs.is_empty())
            .then(|| prelude_origin(&parts.join("::")))
            .flatten();
        if let Some(origin) = prelude {
            return Ok(Located::External(origin));
        }
        let generated_here = self.generated.contains(&module);
        if generated_here {
            return Ok(Located::Item(module));
        }
        if let Some(target) = glob_module(&globs, |target| self.generated.contains(target)) {
            return self.reexport(&module, Located::Item(target), &parts[1..], query);
        }
        if let Some(path) = external_glob(&globs, name) {
            let found = Located::External(path);
            return self.reexport(&module, found, &parts[1..], query);
        }
        bail!(
            "unresolved Rust name {}; imported, generated or lexical bindings require analysis",
            [module.as_slice(), &[name.to_owned()]].concat().join("::")
        );
    }

    // DECISION: D136
    pub(super) fn glob_sources(
        &self,
        scope: &[String],
        query: &mut Query<'_>,
    ) -> Result<Vec<Located>> {
        let mut key = scope.to_vec();
        key.push("*".into());
        let cyclic = !query.visiting.insert(key.clone());
        if cyclic {
            return Ok(Vec::new());
        }
        let mut sources = Vec::new();
        for glob in self.globs.get(scope).into_iter().flatten() {
            let found = self.path(scope, glob, query)?;
            let nested = match &found {
                Located::Module(target) => self.glob_sources(target, query)?,
                Located::Item(_) | Located::External(_) => Vec::new(),
            };
            sources.push(found);
            sources.extend(nested);
        }
        query.visiting.remove(&key);
        Ok(sources)
    }

    pub(super) fn binds(&self, module: &[String], name: &str) -> bool {
        let mut key = module.to_vec();
        key.push(name.into());
        self.modules.contains_key(&key)
            || self.items.contains(&key)
            || self.imports.contains_key(&key)
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
        self.reexport(owner, imported?, suffix, query)
    }

    fn reexport(
        &self,
        owner: &[String],
        imported: Located,
        suffix: &[&str],
        query: &mut Query<'_>,
    ) -> Result<Located> {
        match imported {
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
        self.external.contains(name) && !self.binds(scope, name)
    }

    fn external_path(&self, path: &str, first: &str) -> Result<Located> {
        ensure!(
            self.external.contains(first),
            "undeclared external Rust crate {first}"
        );
        Ok(Located::External(path.into()))
    }
}

// DECISION: D136
fn external_glob(globs: &[Located], name: &str) -> Option<String> {
    let external: Vec<_> = globs
        .iter()
        .filter_map(|glob| match glob {
            Located::External(path) => Some(format!("{path}::{name}")),
            Located::Module(_) | Located::Item(_) => None,
        })
        .collect();
    let [path] = external.as_slice() else {
        return None;
    };
    Some(path.clone())
}

fn glob_module(globs: &[Located], bound: impl Fn(&[String]) -> bool) -> Option<Vec<String>> {
    globs.iter().find_map(|glob| match glob {
        Located::Module(target) if bound(target) => Some(target.clone()),
        Located::Module(_) | Located::Item(_) | Located::External(_) => None,
    })
}
