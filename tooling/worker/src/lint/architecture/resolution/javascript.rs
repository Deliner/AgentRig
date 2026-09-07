//! Relative Node module resolution and TypeScript bundler source resolution.
//! Package imports/aliases need explicit external declarations until their local
//! mappings are supported; unsupported specifiers never imply an absent edge.
#[cfg(test)]
mod tests;

use super::{Resolved, normalize};
use crate::lint::architecture::source::{Loader, Target};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Node,
    TypeScriptBundler,
}

pub struct JavaScript<'a> {
    root: &'a Path,
    files: &'a BTreeSet<PathBuf>,
    external: &'a BTreeSet<String>,
    mode: Mode,
}

impl<'a> JavaScript<'a> {
    /// Canonical root and normalized inventory are owned by lint's collector.
    pub fn new(
        root: &'a Path,
        files: &'a BTreeSet<PathBuf>,
        external: &'a BTreeSet<String>,
        mode: Mode,
    ) -> Self {
        Self {
            root,
            files,
            external,
            mode,
        }
    }

    pub fn resolve(&self, source: &Path, target: &Target) -> Result<Resolved> {
        ensure!(
            self.files.contains(source),
            "source is missing from inventory"
        );
        let Target::JavaScriptModule { path, loader } = target else {
            bail!("not a JavaScript/TypeScript module reference");
        };
        let relative = path.starts_with("./")
            || path.starts_with("../")
            || matches!(path.as_str(), "." | "..");
        if relative {
            ensure!(
                !path.contains(['\\', '%', '?', '#']),
                "encoded or URL module specifier requires URL resolution: {path}"
            );
            let parent = source.parent().context("source has no parent")?;
            let directory = path.ends_with('/')
                || path.ends_with("/.")
                || path.ends_with("/..")
                || matches!(path.as_str(), "." | "..");
            let path = normalize(&parent.join(path))?;
            let target = self.local(&path, *loader, directory)?;
            return Ok(Resolved {
                files: BTreeSet::from([target]),
                external: None,
            });
        }
        self.resolve_external(path)
    }

    fn resolve_external(&self, path: &str) -> Result<Resolved> {
        let external = self
            .external
            .iter()
            .any(|name| path == name || path.starts_with(&format!("{name}/")));
        ensure!(
            external,
            "unresolved package/alias {path}; local package mappings require analysis or an external declaration"
        );
        Ok(Resolved {
            files: BTreeSet::new(),
            external: Some(path.into()),
        })
    }

    fn local(&self, path: &Path, loader: Loader, directory: bool) -> Result<PathBuf> {
        let exact_import = self.mode == Mode::Node && loader == Loader::Import;
        if exact_import {
            ensure!(
                !directory && self.files.contains(path),
                "Node import requires an existing explicit file: {}",
                path.display()
            );
            return Ok(path.into());
        }
        let file = (!directory).then(|| self.file(path)).flatten();
        if let Some(file) = file {
            return Ok(file);
        }
        self.directory(path)?
            .with_context(|| format!("unresolved module {}", path.display()))
    }

    fn file(&self, path: &Path) -> Option<PathBuf> {
        let typescript = self.mode == Mode::TypeScriptBundler;
        if typescript {
            return self.typescript(path);
        }
        let exact = self.files.contains(path);
        if exact {
            return Some(path.into());
        }
        self.append(path, &["js", "json", "node"])
    }

    fn typescript(&self, path: &Path) -> Option<PathBuf> {
        let extensions: &[&str] = match path.extension().and_then(|ext| ext.to_str()) {
            Some("js" | "jsx") => &["ts", "tsx", "d.ts", "js", "jsx"],
            Some("mjs") => &["mts", "d.mts", "mjs"],
            Some("cjs") => &["cts", "d.cts", "cjs"],
            None => return self.append(path, &["ts", "tsx", "d.ts", "js", "jsx"]),
            _ => return self.files.contains(path).then(|| path.into()),
        };
        extensions
            .iter()
            .map(|extension| path.with_extension(extension))
            .find(|candidate| self.files.contains(candidate))
    }

    fn append(&self, path: &Path, suffixes: &[&str]) -> Option<PathBuf> {
        suffixes
            .iter()
            .map(|suffix| PathBuf::from(format!("{}.{suffix}", path.display())))
            .find(|candidate| self.files.contains(candidate))
    }

    fn directory(&self, path: &Path) -> Result<Option<PathBuf>> {
        let manifest = path.join("package.json");
        let main = self
            .files
            .contains(&manifest)
            .then(|| self.entrypoint(&manifest))
            .transpose()?
            .flatten();
        if let Some(main) = main {
            let main = normalize(&path.join(main))?;
            let resolved = self.file(&main).or_else(|| self.index(&main));
            if let Some(resolved) = resolved {
                return Ok(Some(resolved));
            }
            ensure!(
                self.mode == Mode::Node,
                "TypeScript package entry is unresolved; fallback requires analysis: {}",
                manifest.display()
            );
        }
        Ok(self.index(path))
    }

    fn index(&self, path: &Path) -> Option<PathBuf> {
        let suffixes: &[&str] = match self.mode {
            Mode::Node => &["js", "json", "node"],
            Mode::TypeScriptBundler => &["ts", "tsx", "d.ts", "js", "jsx"],
        };
        self.append(&path.join("index"), suffixes)
    }

    fn entrypoint(&self, manifest: &Path) -> Result<Option<String>> {
        let absolute = self.root.join(manifest).canonicalize()?;
        ensure!(
            absolute.starts_with(self.root),
            "package manifest escapes project root"
        );
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&absolute)?)
            .with_context(|| format!("invalid package manifest {}", manifest.display()))?;
        let object = value
            .as_object()
            .context("package manifest must be an object")?;
        let versioned_types =
            self.mode == Mode::TypeScriptBundler && object.contains_key("typesVersions");
        ensure!(
            !versioned_types,
            "typesVersions requires compiler-version-aware resolution"
        );
        let fields: &[&str] = match self.mode {
            Mode::Node => &["main"],
            Mode::TypeScriptBundler => &["types", "typings", "main"],
        };
        for field in fields {
            if let Some(value) = object.get(*field) {
                let entry = value.as_str().context("package entry must be a string")?;
                let nonempty = !entry.is_empty();
                if nonempty {
                    return Ok(Some(entry.into()));
                }
            }
        }
        Ok(None)
    }
}
