// DECISION: D033
mod branches;
mod directories;
mod export;
mod git;
mod hooks;
mod integration;
mod observation;
#[cfg(test)]
mod tests;
pub use git::context_text as git_context;
pub use hooks::{Generated, Generation, generate};
pub use observation::Observation;

use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub base: String,
    pub prefix: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            base: "main".into(),
            prefix: "feature/".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FileKind {
    File,
    Executable,
    Symlink,
    Submodule,
}

impl FileKind {
    pub fn mode(self) -> &'static str {
        match self {
            Self::File => "100644",
            Self::Executable => "100755",
            Self::Symlink => "120000",
            Self::Submodule => "160000",
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub kind: FileKind,
    pub object: String,
}

pub struct Repository<'a> {
    root: &'a Path,
}

pub fn initialize(root: &Path, base: &str) -> Result<()> {
    let missing = Repository::discover(root)?.is_none();
    if missing {
        git::run(root, &["init", "-q", "-b", base])?;
    }
    Ok(())
}

impl<'a> Repository<'a> {
    pub fn discover(root: &'a Path) -> Result<Option<Self>> {
        Ok(root.join(".git").exists().then(|| Self::new(root)))
    }

    pub fn new(root: &'a Path) -> Self {
        Self { root }
    }

    pub fn commit_context(&self) -> Result<(String, bool)> {
        git::commit_context(self.root)
    }

    pub fn resolve(&self, reference: &str) -> Result<String> {
        validate_revision(git::resolve(self.root, reference)?)
    }

    pub fn head(&self) -> Result<Option<String>> {
        Ok(git::head(self.root)?
            .map(validate_revision)
            .transpose()?
            .filter(|value| value.bytes().any(|byte| byte != b'0')))
    }

    pub fn parents(&self, revision: &str) -> Result<Vec<String>> {
        git::parents(self.root, revision)?
            .into_iter()
            .map(validate_revision)
            .collect()
    }

    pub fn tree(&self, revision: &str) -> Result<BTreeMap<String, Entry>> {
        git::tree(self.root, revision)
    }

    pub fn read(&self, revision: &str, path: &str) -> Result<Vec<u8>> {
        git::run(
            self.root,
            &["cat-file", "blob", &format!("{revision}:{path}")],
        )
    }

    pub fn changed_paths(&self, base: &str, candidate: &str) -> Result<Vec<String>> {
        paths(&git::changes(self.root, base, candidate)?)
    }

    pub fn working_files(&self) -> Result<Vec<String>> {
        paths(&git::working_files(self.root)?)
    }

    pub fn staged_changes(&self) -> Result<Vec<String>> {
        paths(&git::run(
            self.root,
            &[
                "diff",
                "--cached",
                "--name-only",
                "--no-renames",
                "-z",
                "--",
            ],
        )?)
    }

    pub fn staged_files(&self) -> Result<Vec<String>> {
        paths(&git::staged_files(self.root)?)
    }

    pub fn index_entries(&self) -> Result<Vec<u8>> {
        git::index_entries(self.root)
    }

    pub fn export_index(&self) -> Result<tempfile::TempDir> {
        git::export_index(self.root)
    }

    pub fn export_revision(&self, reference: &str) -> Result<tempfile::TempDir> {
        let revision = self.resolve(reference)?;
        export::revision(self, &revision)
    }

    pub fn diff(&self, base: &str, candidate: &str) -> Result<String> {
        let bytes = git::run(
            self.root,
            &[
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--binary",
                "--no-renames",
                base,
                candidate,
                "--",
            ],
        )?;
        Ok(String::from_utf8(bytes)?)
    }
}

fn validate_revision(value: String) -> Result<String> {
    let single_revision =
        matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit());
    ensure!(
        single_revision,
        "VCS reference must resolve to one full revision ID"
    );
    Ok(value)
}

fn paths(bytes: &[u8]) -> Result<Vec<String>> {
    let mut paths = bytes
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| Ok(String::from_utf8(name.to_vec())?))
        .collect::<Result<Vec<_>>>()?;
    paths.sort();
    paths.dedup();
    Ok(paths)
}

pub use git::run as git_command;
