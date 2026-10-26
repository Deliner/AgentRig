use super::{Repository, git};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub branch: String,
    pub revision: String,
    pub status: String,
    pub merge_in_progress: bool,
    pub rebase_in_progress: bool,
}

impl Repository<'_> {
    pub fn observe(&self) -> Result<Observation> {
        let branch = String::from_utf8(git::run(self.root, &["branch", "--show-current"])?)?;
        let status = String::from_utf8(git::run(self.root, &["status", "--short"])?)?;
        let merge_in_progress = self.git_operation("MERGE_HEAD")?;
        let rebase_in_progress =
            self.git_operation("rebase-merge")? || self.git_operation("rebase-apply")?;
        Ok(Observation {
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
