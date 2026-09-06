mod merge;
mod source;
#[cfg(test)]
mod tests;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Serialize)]
pub struct Resolved {
    pub configuration: Map<String, Value>,
    pub provenance: BTreeMap<String, PathBuf>,
    pub packages: Vec<Package>,
    pub root: PathBuf,
    pub root_digest: String,
}
impl Resolved {
    pub fn origin(&self, address: &str) -> &Path {
        let mut candidate = address;
        loop {
            if let Some(origin) = self.provenance.get(candidate) {
                return origin;
            }
            let Some((parent, _)) = candidate.rsplit_once('/') else {
                return &self.root;
            };
            candidate = parent;
        }
    }
}

#[derive(Serialize)]
pub struct Package {
    pub id: String,
    pub version: String,
    pub path: PathBuf,
    pub digest: String,
}

#[derive(Default)]
struct Resolver {
    values: merge::Values,
    packages: Vec<Package>,
    visiting: Vec<PathBuf>,
    applied: BTreeSet<(PathBuf, String)>,
}

pub fn resolve(path: &Path) -> Result<Resolved> {
    let path = path
        .canonicalize()
        .with_context(|| format!("resolve {}", path.display()))?;
    let (root, source): (source::Root, _) = review_runner::config::yaml::read_document(&path)?;
    let mut resolver = Resolver::default();
    resolver.imports(&path, &root.packages, "")?;
    resolver
        .values
        .add(&path, root.configuration, &root.overrides)?;
    Ok(Resolved {
        configuration: resolver.values.configuration,
        provenance: resolver.values.provenance,
        packages: resolver.packages,
        root: path,
        root_digest: digest(&source),
    })
}

pub fn cli(root: &Path, args: &[String]) -> Result<i32> {
    ensure!(args.len() == 1, "config-resolve CONFIG_YAML");
    println!(
        "{}",
        serde_json::to_string_pretty(&resolve(&root.join(&args[0]))?)?
    );
    Ok(0)
}

impl Resolver {
    fn imports(
        &mut self,
        declaring: &Path,
        imports: &[source::Import],
        target: &str,
    ) -> Result<()> {
        for reference in imports {
            ensure!(
                reference.into.is_empty() || reference.into.starts_with('/'),
                "package into must be a JSON pointer"
            );
            let path = declaring
                .parent()
                .context("configuration directory required")?
                .join(&reference.path);
            let target = format!("{target}{}", reference.into);
            self.package(&path, reference, &target).with_context(|| {
                format!(
                    "package {} imported by {}",
                    path.display(),
                    declaring.display()
                )
            })?;
        }
        Ok(())
    }

    fn package(&mut self, path: &Path, reference: &source::Import, target: &str) -> Result<()> {
        let path = path.canonicalize()?;
        ensure!(
            !self.visiting.contains(&path),
            "package cycle: {:?} -> {}",
            self.visiting,
            path.display()
        );
        let (package, source): (source::Package, _) =
            review_runner::config::yaml::read_document(&path)?;
        package.validate(reference)?;
        let digest = digest(&source);
        self.known(&path, &package.id, &digest)?;
        let application = (path.clone(), target.to_owned());
        let applied = self.applied.contains(&application);
        if applied {
            return Ok(());
        }
        self.visiting.push(path.clone());
        self.imports(&path, &package.packages, target)?;
        let known = self.known(&path, &package.id, &digest)?;
        self.values
            .add_at(&path, package.configuration, &package.overrides, target)?;
        self.visiting.pop();
        let unseen = !known;
        if unseen {
            self.packages.push(Package {
                id: package.id,
                version: package.version,
                path,
                digest,
            });
        }
        self.applied.insert(application);
        Ok(())
    }

    fn known(&self, path: &Path, id: &str, digest: &str) -> Result<bool> {
        let Some(existing) = self.packages.iter().find(|item| item.id == id) else {
            return Ok(false);
        };
        ensure!(
            existing.path == path && existing.digest == digest,
            "package identity conflict for {id}: {} and {}",
            existing.path.display(),
            path.display()
        );
        Ok(true)
    }
}

fn digest(source: &str) -> String {
    format!("{:x}", Sha256::digest(source.as_bytes()))
}
