use super::{Located, Query, Resolved, Rust};
use crate::lint::architecture::source::RustMacro;
use anyhow::{Context, Result, bail, ensure};
use std::{collections::BTreeSet, path::Path};

impl Rust<'_> {
    pub(super) fn validate_expansions(&self) -> Result<()> {
        for file in self.files.keys() {
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

    pub(super) fn derive_result(
        &self,
        source: &Path,
        path: &str,
        scope: &[String],
    ) -> Result<Resolved> {
        let origin = self.macro_origin(source, path, scope)?;
        self.require_external_macro(&origin)?;
        let known = origin.split_once("::").is_some_and(|(namespace, name)| {
            matches!(namespace, "std" | "core") && derive(name)
                || namespace == "serde" && matches!(name, "Serialize" | "Deserialize")
        });
        ensure!(known, "Rust derive expansion is not analyzed: {origin}");
        Ok(Resolved {
            files: BTreeSet::new(),
            external: Some(origin),
        })
    }

    pub(super) fn macro_result(&self, source: &Path, call: &RustMacro) -> Result<Resolved> {
        let scope = super::joined(&self.files[source], &call.scope);
        let origin = self.macro_origin(source, &call.path, &scope)?;
        self.require_external_macro(&origin)?;
        let known = origin.split_once("::").is_some_and(|(namespace, name)| {
            matches!(namespace, "std" | "core" | "alloc") && standard(name)
                || namespace == "anyhow" && matches!(name, "anyhow" | "bail" | "ensure")
                || namespace == "serde_json" && name == "json"
        });
        ensure!(known, "Rust macro expansion is not analyzed: {origin}");
        let resource = matches!(
            origin.as_str(),
            "std::include_str" | "core::include_str" | "std::include_bytes" | "core::include_bytes"
        );
        let files = if resource {
            let literal = call
                .literal
                .as_ref()
                .context("resource macro needs one static unescaped string literal")?;
            let target = super::super::normalize(&source.parent().unwrap().join(literal))?;
            BTreeSet::from([target])
        } else {
            BTreeSet::new()
        };
        Ok(Resolved {
            files,
            external: Some(origin),
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

    fn macro_origin(&self, source: &Path, path: &str, scope: &[String]) -> Result<String> {
        let mut key = scope.to_vec();
        key.push(path.into());
        let prelude = (standard(path) || derive(path)) && !self.imports.contains_key(&key);
        let wildcard = self.sources[source].rust_imports.iter().any(|import| {
            import.name == "*" && super::joined(&self.files[source], &import.scope) == scope
        });
        ensure!(
            !prelude || !wildcard,
            "Rust macro origin is ambiguous through a wildcard import: {path}"
        );
        let origin = if prelude {
            format!("std::{path}")
        } else {
            let mut query = Query {
                source,
                visiting: BTreeSet::new(),
            };
            match self.path(scope, path, &mut query)? {
                Located::External(origin) => origin,
                Located::Module(_) if (standard(path) || derive(path)) && !wildcard => {
                    format!("std::{path}")
                }
                _ => bail!("Rust local macro expansion is not analyzed: {path}"),
            }
        };
        Ok(origin)
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
    )
}
