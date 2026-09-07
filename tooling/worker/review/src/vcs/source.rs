use super::{Entry, Kind, Repository, external::Adapter};
use anyhow::Result;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, MapAccess, Visitor},
};
use std::{collections::BTreeMap, path::Path};

/// Serializable source selection; native values retain their existing YAML form.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Backend {
    Native(Kind),
    External(Adapter),
}

impl<'de> Deserialize<'de> for Backend {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(BackendVisitor)
    }
}

struct BackendVisitor;

impl<'de> Visitor<'de> for BackendVisitor {
    type Value = Backend;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("git, mercurial, or an external adapter command object")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Backend, E> {
        Kind::deserialize(de::value::StrDeserializer::<E>::new(value)).map(Backend::Native)
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Backend, A::Error> {
        Adapter::deserialize(de::value::MapAccessDeserializer::new(map)).map(Backend::External)
    }
}

impl Default for Backend {
    fn default() -> Self {
        Kind::Git.into()
    }
}

impl From<Kind> for Backend {
    fn from(kind: Kind) -> Self {
        Self::Native(kind)
    }
}

impl Backend {
    pub fn repository_present(&self, root: &Path) -> Result<bool> {
        match self {
            Self::Native(kind) => Ok(kind.repository(root)?.is_some()),
            Self::External(adapter) => {
                adapter.call(root, "repository-present", serde_json::json!({}))
            }
        }
    }

    pub fn initialize(&self, root: &Path, base: &str) -> Result<()> {
        match self {
            Self::Native(kind) => kind.initialize(root, base),
            Self::External(adapter) => adapter.initialize(root, base),
        }
    }

    pub fn native(&self, operation: &str) -> Result<Kind> {
        match self {
            Self::Native(kind) => Ok(*kind),
            Self::External(_) => anyhow::bail!(
                "private VCS {operation} is not implemented; configured reads remain available"
            ),
        }
    }

    pub fn executable(&self) -> &str {
        match self {
            Self::Native(kind) => kind.executable(),
            Self::External(adapter) => adapter.command.first().map(String::as_str).unwrap_or(""),
        }
    }

    pub fn repository<'a>(&self, root: &'a Path) -> Result<Option<Repository<'a>>> {
        self.native("setup and delivery")?.repository(root)
    }

    pub fn repository_source<'a>(&'a self, root: &'a Path) -> Result<Option<Source<'a>>> {
        let present = match self {
            Self::Native(kind) => kind.repository(root)?.is_some(),
            Self::External(_) => true,
        };
        Ok(present.then(|| self.source(root)))
    }

    pub fn matches_previous(&self, root: &Path, previous: (&Self, &Path)) -> bool {
        // Native hashes identify content globally; external IDs may be repository-local.
        self == previous.0 && (matches!(self, Self::Native(_)) || root == previous.1)
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Native(_) => Ok(()),
            Self::External(adapter) => adapter.validate(),
        }
    }

    pub fn source<'a>(&'a self, root: &'a Path) -> Source<'a> {
        Source {
            root,
            backend: self,
        }
    }
}

pub struct Source<'a> {
    pub(super) root: &'a Path,
    pub(super) backend: &'a Backend,
}

impl Source<'_> {
    pub fn observe(&self) -> Result<super::Observation> {
        match self.backend {
            Backend::Native(kind) => Repository::new(self.root, *kind).observe(),
            Backend::External(adapter) => adapter.observe(self.root),
        }
    }

    /// Export committed project inputs for delivery checks, preserving file kinds.
    /// Isolated review must use its restricted snapshot exporter instead.
    pub fn export_revision(&self, reference: &str) -> Result<tempfile::TempDir> {
        let revision = self.resolve(reference)?;
        super::export::revision(self, &revision)
    }

    pub fn resolve(&self, reference: &str) -> Result<String> {
        match self.backend {
            Backend::Native(kind) => Repository::new(self.root, *kind).resolve(reference),
            Backend::External(adapter) => adapter.resolve(self.root, reference),
        }
    }

    pub fn head(&self) -> Result<Option<String>> {
        match self.backend {
            Backend::Native(kind) => Repository::new(self.root, *kind).head(),
            Backend::External(adapter) => adapter.head(self.root),
        }
    }

    pub fn parents(&self, revision: &str) -> Result<Vec<String>> {
        match self.backend {
            Backend::Native(kind) => Repository::new(self.root, *kind).parents(revision),
            Backend::External(adapter) => adapter.parents(self.root, revision),
        }
    }

    pub fn tree(&self, revision: &str) -> Result<BTreeMap<String, Entry>> {
        match self.backend {
            Backend::Native(kind) => Repository::new(self.root, *kind).tree(revision),
            Backend::External(adapter) => adapter.tree(self.root, revision),
        }
    }

    pub fn read(&self, revision: &str, path: &str) -> Result<Vec<u8>> {
        match self.backend {
            Backend::Native(kind) => Repository::new(self.root, *kind).read(revision, path),
            Backend::External(adapter) => adapter.read(self.root, revision, path),
        }
    }

    pub fn changed_paths(&self, base: &str, candidate: &str) -> Result<Vec<String>> {
        match self.backend {
            Backend::Native(kind) => {
                Repository::new(self.root, *kind).changed_paths(base, candidate)
            }
            Backend::External(adapter) => adapter.changed_paths(self.root, (base, candidate)),
        }
    }

    pub fn working_files(&self) -> Result<Vec<String>> {
        match self.backend {
            Backend::Native(kind) => Repository::new(self.root, *kind).working_files(),
            Backend::External(adapter) => adapter.working_files(self.root),
        }
    }

    pub fn diff(&self, base: &str, candidate: &str) -> Result<String> {
        match self.backend {
            Backend::Native(kind) => Repository::new(self.root, *kind).diff(base, candidate),
            Backend::External(adapter) => adapter.diff(self.root, (base, candidate)),
        }
    }
}
