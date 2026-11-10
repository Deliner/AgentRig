use super::{Located, Query, Resolved, Rust};
use crate::structural_lint::architecture::resolution::normalize;
use crate::structural_lint::architecture::source::RustMacro;
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

impl Rust<'_> {
    pub(super) fn validate_expansions(&self) -> Result<()> {
        for file in self.files.keys().chain(self.included.keys()) {
            for reference in &self.sources[file].items {
                let expansion = matches!(
                    reference.target,
                    super::Target::RustMacro(_) | super::Target::RustDerive { .. }
                );
                if expansion {
                    self.resolve(file, &reference.target)?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn derive_result(&self, path: &str, scope: &[String]) -> Result<Resolved> {
        let origin = self.macro_origin(path, scope)?;
        self.require_external_macro(&origin)?;
        let known = origin.split_once("::").is_some_and(|(namespace, name)| {
            let standard_derive = matches!(namespace, "std" | "core") && derive(name);
            let serde_derive = namespace == "serde" && matches!(name, "Serialize" | "Deserialize");
            standard_derive || serde_derive
        });
        ensure!(known, "Rust derive expansion is not analyzed: {origin}");
        Ok(Resolved {
            files: BTreeSet::new(),
            external: Some(origin),
        })
    }

    // DECISION: D136
    pub(super) fn macro_result(
        &self,
        source: &Path,
        base: &[String],
        call: &RustMacro,
    ) -> Result<Resolved> {
        let scope = super::joined(base, &call.scope);
        if let Some(file) = self.local_macro(&scope, &call.path) {
            return Ok(Resolved {
                files: BTreeSet::from([file.to_path_buf()]),
                external: None,
            });
        }
        let origin = self.macro_origin(&call.path, &scope)?;
        self.require_external_macro(&origin)?;
        let resource = matches!(
            origin.as_str(),
            "std::include_str" | "core::include_str" | "std::include_bytes" | "core::include_bytes"
        );
        let files = if resource {
            let literal = call
                .literal
                .as_ref()
                .context("resource macro needs one static unescaped string literal")?;
            let parent = source.parent().context("resource source has no parent")?;
            let target = normalize(&parent.join(literal))?;
            BTreeSet::from([target])
        } else {
            BTreeSet::new()
        };
        Ok(Resolved {
            files,
            external: Some(origin),
        })
    }

    // DECISION: D136
    fn local_macro(&self, scope: &[String], name: &str) -> Option<&Path> {
        let qualified = name.contains("::");
        if qualified {
            return None;
        }
        (0..=scope.len()).rev().find_map(|depth| {
            let mut key = scope[..depth].to_vec();
            key.push(name.into());
            self.macros.get(&key).map(PathBuf::as_path)
        })
    }

    fn require_external_macro(&self, origin: &str) -> Result<()> {
        let namespace = origin.split("::").next().unwrap_or(origin);
        ensure!(
            !self.local.contains(namespace),
            "local crate macro expansion is not analyzed: {origin}"
        );
        Ok(())
    }

    fn macro_origin(&self, path: &str, scope: &[String]) -> Result<String> {
        let mut key = scope.to_vec();
        key.push(path.into());
        let module = self
            .modules
            .get(scope)
            .context("Rust module scope is unresolved")?;
        let mut query = Query {
            source: &module.file,
            visiting: BTreeSet::new(),
        };
        let builtin = standard(path) || derive(path);
        // DECISION: D136
        let prelude = builtin
            && !self.imports.contains_key(&key)
            && !self
                .glob_sources(scope, &mut query)?
                .iter()
                .any(|glob| matches!(glob, Located::Module(target) if self.binds(target, path)));
        if prelude {
            return Ok(format!("std::{path}"));
        }
        match self.path(scope, path, &mut query)? {
            Located::External(origin) => Ok(origin),
            Located::Module(_) if builtin => Ok(format!("std::{path}")),
            Located::Module(_) | Located::Item(_) => {
                bail!("Rust local macro expansion is not analyzed: {path}")
            }
        }
    }
}

fn derive(name: &str) -> bool {
    matches!(
        name,
        "Clone" | "Copy" | "Debug" | "Default" | "Eq" | "Hash" | "Ord" | "PartialEq" | "PartialOrd"
    )
}

fn standard(name: &str) -> bool {
    matches!(
        name,
        "assert"
            | "assert_eq"
            | "assert_ne"
            | "debug_assert"
            | "debug_assert_eq"
            | "debug_assert_ne"
            | "format"
            | "format_args"
            | "print"
            | "println"
            | "eprint"
            | "eprintln"
            | "write"
            | "writeln"
            | "vec"
            | "matches"
            | "panic"
            | "unreachable"
            | "todo"
            | "unimplemented"
            | "cfg"
            | "env"
            | "option_env"
            | "include_str"
            | "include_bytes"
            | "column"
            | "compile_error"
            | "concat"
            | "dbg"
            | "file"
            | "line"
            | "module_path"
            | "stringify"
            | "thread_local"
    )
}
