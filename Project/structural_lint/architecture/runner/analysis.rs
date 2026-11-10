use crate::shared::fs;
use crate::shared::paths::patterns::globs;
use crate::structural_lint::architecture::{
    self as architecture, Dependency, EnvironmentUse, Issue, Settings,
    resolution::{Resolved, python::Python, rust::Rust},
    source::{self, Reference, References, Target},
};
use crate::structural_lint::rules::descriptor::TEST_CODE;
use anyhow::{Result, ensure};
use globset::GlobSet;
use std::{
    collections::{BTreeMap, BTreeSet},
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
    let file = fs::canonicalize(root.join(path))?;
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
            Target::RustPath { .. }
            | Target::RustMacro(_)
            | Target::RustDerive { .. }
            | Target::RustModule { .. }
            | Target::RustInclude { .. } => (
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
    disallowed: super::disallowed::Policy<'a>,
    // DECISION: D146
    test_code: GlobSet,
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
                disallowed: super::disallowed::Policy::new(&settings.disallowed)?,
                test_code: globs(&TEST_CODE.iter().map(|&p| p.into()).collect::<Vec<_>>())?,
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
            Target::RustPath { .. }
            | Target::RustDerive { .. }
            | Target::RustModule { .. }
            | Target::RustInclude { .. } => self.rust(path, target),
        }?;
        for file in &resolved.files {
            ensure!(
                fs::canonicalize(self.root.join(file))?.starts_with(self.root),
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
        let mut gaps = references.issues;
        let mut violations = Vec::new();
        let mut edges = Vec::new();
        let mut used = Externals::default();
        let mut environment = Vec::new();
        for reference in references.items {
            match self.resolve(path, &reference.target) {
                Ok(resolved) => {
                    if let Some(external) = &resolved.external {
                        used.record(&reference.target, external);
                        environment.extend(environment_use(reference.line, external));
                        // DECISION: D105
                        violations.extend(self.disallowed.check(path, reference.line, external));
                    }
                    edges.extend(self.edges(path, &reference, resolved.files));
                }
                Err(error) => gaps.push(incomplete(path, Some(reference.line), error)),
            }
        }
        // DECISION: D125
        violations.extend(self.disallowed.methods(path, &references.propagated_calls));
        Dependencies {
            edges,
            incomplete: gaps,
            violations,
            used,
            environment,
        }
    }

    // DECISION: D136
    // DECISION: D146
    fn edges(
        &self,
        path: &Path,
        reference: &Reference,
        files: BTreeSet<PathBuf>,
    ) -> impl Iterator<Item = Dependency> {
        let relation = reference.target.relation();
        let test = reference.test || self.test_code.is_match(path);
        let (source, line) = (path.to_path_buf(), reference.line);
        files.into_iter().map(move |target| Dependency {
            source: source.clone(),
            target,
            line,
            relation,
            test,
        })
    }
}

// DECISION: D078
// DECISION: D083
// DECISION: D104
fn environment_use(line: usize, external: &str) -> Option<EnvironmentUse> {
    architecture::effect(external).map(|_| EnvironmentUse {
        line,
        path: external.trim_start_matches("::").to_owned(),
    })
}

// DECISION: D141
pub struct Dependencies {
    pub edges: Vec<Dependency>,
    pub incomplete: Vec<Issue>,
    pub violations: Vec<Issue>,
    pub used: Externals,
    pub environment: Vec<EnvironmentUse>,
}
