use super::format::{self, Catalog, Row};
use crate::project_config::{Context, FILE};
use crate::shared::vcs::{Entry, Repository};
use anyhow::{Context as _, Result, ensure};
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

pub(super) struct Snapshot<'a> {
    repository: &'a Repository<'a>,
    tree: BTreeMap<String, Entry>,
    memory: String,
    read: HashMap<String, String>,
}

impl<'a> Snapshot<'a> {
    pub(super) fn load(
        context: &Context,
        repository: &'a Repository<'a>,
        revision: &str,
    ) -> Result<Self> {
        let tree = repository.tree(revision)?;
        let memory = directory(context, repository, revision, &tree)?;
        Ok(Self {
            repository,
            tree,
            memory,
            read: HashMap::new(),
        })
    }

    pub(super) fn path(&self, name: &str) -> String {
        format!("{}/{name}", self.memory)
    }

    pub(super) fn read_ahead(&mut self, paths: &[String]) -> Result<()> {
        let unread: BTreeMap<&String, &str> = paths
            .iter()
            .filter(|path| !self.read.contains_key(*path))
            .filter_map(|path| Some((path, self.tree.get(path)?.object.as_str())))
            .collect();
        let nothing_unread = unread.is_empty();
        if nothing_unread {
            return Ok(());
        }
        let objects: Vec<&str> = unread.values().copied().collect();
        let mut texts = Vec::new();
        self.repository.blobs(&objects, |bytes| {
            texts.push(String::from_utf8(bytes)?);
            Ok(())
        })?;
        let read = unread.into_keys().cloned().zip(texts);
        self.read.extend(read);
        Ok(())
    }

    pub(super) fn text(&mut self, path: &str) -> Result<Option<String>> {
        self.read_ahead(&[path.to_owned()])?;
        Ok(self.read.get(path).cloned())
    }

    pub(super) fn table(&mut self, name: &str, catalog: Catalog) -> Result<Option<Vec<Row>>> {
        let path = self.path(name);
        self.text(&path)?
            .map(|text| format::parse_table(&text, Path::new(&path), catalog))
            .transpose()
    }

    pub(super) fn located(&self, link: &str) -> Result<String> {
        let joined = self.path(link);
        let mut parts: Vec<&str> = Vec::new();
        for part in joined.split('/') {
            match part {
                "" | "." => {}
                ".." => ensure!(
                    parts.pop().is_some(),
                    "memory link leaves the repository: {link}"
                ),
                _ => parts.push(part),
            }
        }
        Ok(parts.join("/"))
    }
}

fn read(repository: &Repository<'_>, revision: &str, path: &Path) -> Result<String> {
    let bytes = repository
        .read(revision, path)
        .with_context(|| format!("cannot read committed memory {}", path.display()))?;
    Ok(String::from_utf8(bytes)?)
}

fn directory(
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
