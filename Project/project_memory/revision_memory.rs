use crate::project_config::{Context, FILE};
use crate::shared::vcs::{Entry, Repository};
use anyhow::{Context as _, Result};
use std::{collections::BTreeMap, path::Path};

pub(super) fn read(repository: &Repository<'_>, revision: &str, path: &Path) -> Result<String> {
    let bytes = repository
        .read(revision, path)
        .with_context(|| format!("cannot read committed memory {}", path.display()))?;
    Ok(String::from_utf8(bytes)?)
}

pub(super) fn read_all(
    repository: &Repository<'_>,
    tree: &BTreeMap<String, Entry>,
    paths: &[String],
) -> Result<Vec<String>> {
    let objects = paths
        .iter()
        .map(|path| {
            let entry = tree
                .get(path)
                .with_context(|| format!("cannot read committed memory {path}"))?;
            Ok(entry.object.as_str())
        })
        .collect::<Result<Vec<_>>>()?;
    let mut texts = Vec::new();
    repository.blobs(&objects, |bytes| {
        texts.push(String::from_utf8(bytes)?);
        Ok(())
    })?;
    Ok(texts)
}

pub(super) fn directory(
    context: &Context,
    repository: &Repository<'_>,
    revision: &str,
    tree: &BTreeMap<String, Entry>,
) -> Result<String> {
    let configured = tree.contains_key(FILE);
    if configured {
        let prior: serde_json::Value =
            crate::shared::strict_yaml::decode(&read(repository, revision, Path::new(FILE))?)
                .with_context(|| format!("committed {FILE}"))?;
        return Ok(prior["paths"]["memory"]
            .as_str()
            .context("committed paths.memory required")?
            .into());
    }
    Ok(context.config.paths.memory.clone())
}
