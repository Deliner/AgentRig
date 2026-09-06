//! VCS-owned revision and file operations shared by delivery consumers.
mod git;
mod mercurial;

use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    #[default]
    Git,
    Mercurial,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    kind: Kind,
}

impl<'a> Repository<'a> {
    pub fn discover(root: &'a Path) -> Result<Option<Self>> {
        match (root.join(".git").exists(), root.join(".hg").exists()) {
            (true, false) => Ok(Some(Self::new(root, Kind::Git))),
            (false, true) => Ok(Some(Self::new(root, Kind::Mercurial))),
            (false, false) => Ok(None),
            (true, true) => anyhow::bail!("ambiguous VCS root: both .git and .hg exist"),
        }
    }

    pub fn new(root: &'a Path, kind: Kind) -> Self {
        Self { root, kind }
    }

    pub fn resolve(&self, reference: &str) -> Result<String> {
        let value = match self.kind {
            Kind::Git => git::resolve(self.root, reference)?,
            Kind::Mercurial => mercurial::resolve(self.root, reference)?,
        };
        validate_revision(value)
    }

    pub fn head(&self) -> Result<Option<String>> {
        let value = match self.kind {
            Kind::Git => git::head(self.root)?,
            Kind::Mercurial => Some(mercurial::resolve(self.root, ".")?),
        };
        Ok(value
            .map(validate_revision)
            .transpose()?
            .filter(|value| value.bytes().any(|byte| byte != b'0')))
    }

    pub fn tree(&self, revision: &str) -> Result<BTreeMap<String, Entry>> {
        match self.kind {
            Kind::Git => git::tree(self.root, revision),
            Kind::Mercurial => mercurial::tree(self.root, revision),
        }
    }

    pub fn read(&self, revision: &str, path: &str) -> Result<Vec<u8>> {
        match self.kind {
            Kind::Git => git::run(
                self.root,
                &["cat-file", "blob", &format!("{revision}:{path}")],
            ),
            Kind::Mercurial => mercurial::run(
                self.root,
                &["cat", "--rev", revision, "--", &format!("path:{path}")],
            ),
        }
    }

    pub fn changed_paths(&self, base: &str, candidate: &str) -> Result<Vec<String>> {
        let bytes = match self.kind {
            Kind::Git => git::changes(self.root, base, candidate),
            Kind::Mercurial => mercurial::changes(self.root, base, candidate),
        }?;
        paths(&bytes)
    }

    pub fn working_files(&self) -> Result<Vec<String>> {
        let bytes = match self.kind {
            Kind::Git => git::working_files(self.root),
            Kind::Mercurial => mercurial::working_files(self.root),
        }?;
        paths(&bytes)
    }

    pub fn diff(&self, base: &str, candidate: &str) -> Result<String> {
        let bytes = match self.kind {
            Kind::Git => git::run(
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
            ),
            Kind::Mercurial => mercurial::run(
                self.root,
                &["diff", "--git", "--rev", base, "--rev", candidate],
            ),
        }?;
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
