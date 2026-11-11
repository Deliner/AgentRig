// DECISION: D033
mod branches;
mod directories;
mod export;
mod git;
mod hooks;
mod observation;
#[cfg(test)]
mod tests;
pub use git::context_text as git_context;
pub use hooks::{Generated, Generation, generate};
pub use observation::Observation;

use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Copy)]
pub struct Revisions<'a> {
    pub from: &'a str,
    pub to: &'a str,
}

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

    pub fn run(&self, args: &[&str]) -> Result<Vec<u8>> {
        git::run(self.root, args)
    }

    pub fn command(&self, args: &[&str]) -> std::process::Command {
        git::command(self.root, args)
    }

    pub fn ancestor(&self, revisions: Revisions<'_>) -> Result<bool> {
        git::ancestor(self.root, revisions)
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

    pub fn read(&self, revision: &str, path: &Path) -> Result<Vec<u8>> {
        git::run(
            self.root,
            &[
                "cat-file",
                "blob",
                &format!("{revision}:{path}", path = path.display()),
            ],
        )
    }

    pub fn changed_paths(&self, revisions: Revisions<'_>) -> Result<Vec<String>> {
        paths(&git::changes(self.root, revisions)?)
    }

    // DECISION: D140
    pub fn renames(&self, base: &str) -> Result<BTreeMap<String, String>> {
        let arguments = [
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "-M",
            "--name-status",
            "-z",
            base,
            "--",
        ];
        let output = String::from_utf8(git::run(self.root, &arguments)?)?;
        let mut fields = output.split('\0');
        let mut renames = BTreeMap::new();
        while let Some(status) = fields.next().filter(|status| !status.is_empty()) {
            let source = fields.next().unwrap_or_default();
            let renamed = status.starts_with('R');
            if renamed {
                let target = fields.next().unwrap_or_default();
                renames.insert(source.to_owned(), target.to_owned());
            }
        }
        Ok(renames)
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

    pub fn index(&self) -> Result<BTreeMap<String, Entry>> {
        git::index(self.root)
    }

    pub fn export_index(&self) -> Result<tempfile::TempDir> {
        git::export_index(self.root)
    }

    pub fn export_revision(&self, reference: &str) -> Result<tempfile::TempDir> {
        let revision = self.resolve(reference)?;
        export::revision(self, &revision)
    }

    pub fn diff(&self, revisions: Revisions<'_>) -> Result<String> {
        let bytes = git::run(
            self.root,
            &[
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--binary",
                "--no-renames",
                revisions.from,
                revisions.to,
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
