use super::super::{
    Dependency, Issue, Settings,
    resolution::{
        Resolved,
        javascript::{JavaScript, Mode},
        python::Python,
        rust::Rust,
    },
    source::{self, References, Target},
};
use anyhow::{Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub fn extract(root: &Path, files: &BTreeSet<PathBuf>) -> BTreeMap<PathBuf, References> {
    files
        .iter()
        .map(|path| {
            let parsed = read(root, path).unwrap_or_else(|error| References {
                issues: vec![incomplete(path, None, error)],
                ..References::default()
            });
            (path.clone(), parsed)
        })
        .collect()
}

pub fn rust_sources(
    root: &Path,
    files: &BTreeSet<PathBuf>,
    settings: &Settings,
) -> BTreeMap<PathBuf, References> {
    let rust_files = files
        .iter()
        .filter(|path| {
            let rust = path.extension().is_some_and(|ext| ext == "rs");
            rust && settings
                .rust_roots
                .iter()
                .any(|root| root.parent().is_some_and(|parent| path.starts_with(parent)))
        })
        .cloned()
        .collect();
    extract(root, &rust_files)
}

fn read(root: &Path, path: &Path) -> Result<References> {
    let file = root.join(path).canonicalize()?;
    ensure!(file.starts_with(root), "source escapes project root");
    source::extract(path, &fs::read_to_string(file)?)
}

pub fn incomplete(path: &Path, line: Option<usize>, error: anyhow::Error) -> Issue {
    Issue {
        path: path.into(),
        line,
        message: format!("dependency analysis incomplete: {error:#}"),
    }
}

pub struct Resolvers<'a> {
    root: &'a Path,
    files: &'a BTreeSet<PathBuf>,
    python: Python<'a>,
    javascript: JavaScript<'a>,
    typescript: JavaScript<'a>,
    rust: Vec<Rust<'a>>,
    crates: &'a BTreeMap<String, PathBuf>,
}

impl<'a> Resolvers<'a> {
    pub fn new(
        root: &'a Path,
        files: &'a BTreeSet<PathBuf>,
        sources: &'a BTreeMap<PathBuf, References>,
        settings: &'a Settings,
    ) -> Result<(Self, Vec<Issue>)> {
        let mut rust = Vec::new();
        let mut issues = Vec::new();
        for path in &settings.rust_roots {
            match Rust::new(
                path,
                sources,
                &settings.external.rust,
                &settings.rust_crates,
            ) {
                Ok(resolver) => rust.push(resolver),
                Err(error) => issues.push(incomplete(path, None, error)),
            }
        }
        Ok((
            Self {
                root,
                files,
                python: Python::new(&settings.python_root, files, &settings.external.python)?,
                javascript: JavaScript::new(root, files, &settings.external.javascript, Mode::Node),
                typescript: JavaScript::new(
                    root,
                    files,
                    &settings.external.javascript,
                    Mode::TypeScriptBundler,
                ),
                rust,
                crates: &settings.rust_crates,
            },
            issues,
        ))
    }

    pub fn resolve(&self, path: &Path, target: &Target) -> Result<Resolved> {
        let resolved = match target {
            Target::RustMacro(_) => self.rust_macro(path, target),
            Target::PythonModule(_) | Target::PythonFrom { .. } => {
                self.python.resolve(path, target)
            }
            Target::JavaScriptModule { .. } => {
                let typescript = path
                    .extension()
                    .is_some_and(|ext| ["ts", "tsx", "mts", "cts"].iter().any(|item| ext == *item));
                let resolver = if typescript {
                    &self.typescript
                } else {
                    &self.javascript
                };
                resolver.resolve(path, target)
            }
            _ => self.rust(path, target),
        }?;
        for file in &resolved.files {
            ensure!(
                self.root.join(file).canonicalize()?.starts_with(self.root),
                "dependency target escapes project root: {}",
                file.display()
            );
        }
        Ok(resolved)
    }

    fn rust_macro(&self, path: &Path, target: &Target) -> Result<Resolved> {
        let resolved = self.rust(path, target)?;
        for file in &resolved.files {
            ensure!(
                self.files.contains(file),
                "embedded resource is absent from source inventory: {}",
                file.display()
            );
        }
        Ok(resolved)
    }

    fn rust(&self, path: &Path, target: &Target) -> Result<Resolved> {
        let owners: Vec<_> = self
            .rust
            .iter()
            .filter(|resolver| resolver.contains_source(path))
            .collect();
        ensure!(
            !owners.is_empty(),
            "source has no successfully analyzed Rust crate root"
        );
        let mut combined = Resolved::default();
        for resolver in owners {
            combined
                .files
                .extend(self.linked(resolver.resolve(path, target)?)?.files);
        }
        Ok(combined)
    }

    fn linked(&self, mut result: Resolved) -> Result<Resolved> {
        let mut visiting = BTreeSet::new();
        while let Some(reference) = result.external.as_deref() {
            let reference = reference.trim_start_matches("::");
            let (name, suffix) = reference.split_once("::").unwrap_or((reference, ""));
            let Some(root) = self.crates.get(name) else {
                break;
            };
            ensure!(
                visiting.insert(reference.to_owned()),
                "cyclic local Rust crate reference: {reference}"
            );
            let resolver = self
                .rust
                .iter()
                .find(|resolver| resolver.is_root(root))
                .ok_or_else(|| {
                    anyhow::anyhow!("local Rust crate root was not analyzed: {}", root.display())
                })?;
            let path = match suffix.is_empty() {
                true => "crate".into(),
                false => format!("crate::{suffix}"),
            };
            let next = resolver.resolve(
                root,
                &Target::RustPath {
                    path,
                    scope: Vec::new(),
                },
            )?;
            result.files.extend(next.files);
            result.external = next.external;
        }
        Ok(result)
    }

    pub fn dependencies(
        &self,
        path: &Path,
        references: References,
    ) -> (Vec<Dependency>, Vec<Issue>) {
        let mut issues = references.issues;
        let mut edges = Vec::new();
        for reference in references.items {
            match self.resolve(path, &reference.target) {
                Ok(resolved) => edges.extend(resolved.files.into_iter().map(|target| Dependency {
                    source: path.into(),
                    target,
                    line: reference.line,
                })),
                Err(error) => issues.push(incomplete(path, Some(reference.line), error)),
            }
        }
        (edges, issues)
    }
}
