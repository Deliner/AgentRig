use super::{Config, Source};
use anyhow::{Context, Result, ensure};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

impl Source {
    pub(super) fn environment(&mut self, config: &mut Config) -> Result<()> {
        for skill in &mut config.environment.skills {
            let declaring = self.resolved.origin("/environment/skills");
            let source = review_runner::config::resource(declaring, skill)?;
            let name = source
                .file_name()
                .and_then(|name| name.to_str())
                .context("skill name")?;
            let target = format!("{}/{name}", config.frontend.project_skills());
            self.resources.directory_at(&source, &target)?;
            *skill = target.into();
        }
        for (name, program) in &mut config.environment.programs {
            let declaring = self
                .resolved
                .origin(&format!("/environment/programs/{name}"));
            let source = review_runner::config::resource(declaring, program)?;
            *program = self.resources.copy(&source)?.into();
        }
        Ok(())
    }

    pub(super) fn validate_packages(&self) -> Result<()> {
        let configurations = std::iter::once(&self.resolved).chain(self.configurations.values());
        let mut identities =
            std::collections::BTreeMap::<&str, &agentrig::composition::Package>::new();
        for package in configurations.flat_map(|resolved| &resolved.packages) {
            if let Some(existing) = identities.insert(&package.id, package) {
                ensure!(
                    existing.path == package.path && existing.digest == package.digest,
                    "package identity conflict for {} across configurations: {} and {}",
                    package.id,
                    existing.path.display(),
                    package.path.display()
                );
            }
        }
        Ok(())
    }

    pub(super) fn configuration<T: DeserializeOwned>(&mut self, path: &Path) -> Result<T> {
        let resolved = agentrig::composition::resolve(path)?;
        for (path, digest) in std::iter::once((&resolved.root, &resolved.root_digest)).chain(
            resolved
                .packages
                .iter()
                .map(|package| (&package.path, &package.digest)),
        ) {
            ensure!(
                agentrig::resources::digest(&self.resources.read(path)?) == *digest,
                "configuration changed while composing: {}",
                path.display()
            );
        }
        let value = review_runner::config::yaml::decode(&review_runner::config::yaml::encode(
            &resolved.configuration,
        )?)
        .with_context(|| format!("composed configuration {}", path.display()))?;
        self.configurations.insert(path.to_owned(), resolved);
        Ok(value)
    }

    pub(super) fn resource(&self, config: &Path, address: &str, value: &Path) -> Result<PathBuf> {
        let declaring = self.configurations[config].origin(address);
        review_runner::config::resource(declaring, value)
    }
}
