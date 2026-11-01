use super::super::{
    Dependency, EnvironmentUse, Issue, Settings,
    resolution::{Resolved, python::Python, rust::Rust},
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

pub fn rust_sources(root: &Path, files: &BTreeSet<PathBuf>) -> BTreeMap<PathBuf, References> {
    let rust_files = files
        .iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
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

// DECISION: D076
#[derive(Default)]
pub struct Externals {
    pub rust: BTreeSet<String>,
    pub python: BTreeSet<String>,
}

impl Externals {
    fn record(&mut self, target: &Target, external: &str) {
        let (set, first) = match target {
            Target::PythonModule(_) | Target::PythonFrom { .. } => {
                (&mut self.python, external.split('.').next())
            }
            _ => (
                &mut self.rust,
                external.trim_start_matches("::").split("::").next(),
            ),
        };
        if let Some(name) = first.filter(|name| !name.is_empty()) {
            set.insert(name.to_owned());
        }
    }
}

pub struct Resolvers<'a> {
    root: &'a Path,
    files: &'a BTreeSet<PathBuf>,
    python: Python<'a>,
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
            let resolved = resolver.resolve(path, target)?;
            let external = resolved.external.clone();
            combined.files.extend(self.linked(resolved)?.files);
            combined.external = combined.external.take().or(external);
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

    pub fn dependencies(&self, path: &Path, references: References) -> Dependencies {
        let mut issues = references.issues;
        let mut edges = Vec::new();
        let mut used = Externals::default();
        let mut environment = Vec::new();
        for reference in references.items {
            match self.resolve(path, &reference.target) {
                Ok(resolved) => {
                    if let Some(external) = &resolved.external {
                        used.record(&reference.target, external);
                        // DECISION: D078
                        // DECISION: D083
                        let impure =
                            super::super::environment(external) || super::super::clock(external);
                        if impure {
                            environment.push(EnvironmentUse {
                                line: reference.line,
                                path: external.trim_start_matches("::").to_owned(),
                            });
                        }
                    }
                    edges.extend(resolved.files.into_iter().map(|target| Dependency {
                        source: path.into(),
                        target,
                        line: reference.line,
                        module_declaration: matches!(reference.target, Target::RustModule { .. }),
                    }));
                }
                Err(error) => issues.push(incomplete(path, Some(reference.line), error)),
            }
        }
        Dependencies {
            edges,
            issues,
            used,
            environment,
        }
    }
}

pub struct Dependencies {
    pub edges: Vec<Dependency>,
    pub issues: Vec<Issue>,
    pub used: Externals,
    pub environment: Vec<EnvironmentUse>,
}
