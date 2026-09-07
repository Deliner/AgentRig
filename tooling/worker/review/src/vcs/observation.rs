use super::{Backend, Kind, Repository, git, mercurial};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    #[serde(skip_deserializing)]
    pub backend: Backend,
    pub branch: String,
    pub revision: String,
    pub status: String,
    pub merge_in_progress: bool,
    pub rebase_in_progress: bool,
}

impl Repository<'_> {
    pub fn observe(&self) -> Result<Observation> {
        let (branch, status, merge_in_progress, rebase_in_progress) = match self.kind {
            Kind::Git => (
                String::from_utf8(git::run(self.root, &["branch", "--show-current"])?)?,
                String::from_utf8(git::run(self.root, &["status", "--short"])?)?,
                self.git_operation("MERGE_HEAD")?,
                self.git_operation("rebase-merge")? || self.git_operation("rebase-apply")?,
            ),
            Kind::Mercurial => (
                String::from_utf8(mercurial::run(self.root, &["branch"])?)?,
                String::from_utf8(mercurial::run(self.root, &["status"])?)?,
                // Revset queries can populate tag caches; parents observes dirstate directly.
                String::from_utf8(mercurial::run(
                    self.root,
                    &["parents", "--template", "{node}\n"],
                )?)?
                .lines()
                .count()
                    > 1,
                self.root.join(".hg/rebasestate").exists(),
            ),
        };
        Ok(Observation {
            backend: self.kind.into(),
            branch: branch.trim().into(),
            revision: self.head()?.unwrap_or_default(),
            status: status.trim().into(),
            merge_in_progress,
            rebase_in_progress,
        })
    }

    fn git_operation(&self, name: &str) -> Result<bool> {
        let path = String::from_utf8(git::run(self.root, &["rev-parse", "--git-path", name])?)?;
        Ok(self.root.join(path.trim()).exists())
    }
}
