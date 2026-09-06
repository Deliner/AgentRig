use super::Source;
use anyhow::{Context, Result, ensure};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

impl Source {
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
