use super::{Repository, git};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub branch: String,
    pub revision: String,
    pub status: String,
    pub operation: Option<Operation>,
}

// DECISION: D154
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    Merge,
    Rebase,
}

impl Repository<'_> {
    pub fn observe(&self) -> Result<Observation> {
        let branch = String::from_utf8(git::run(self.root, &["branch", "--show-current"])?)?;
        let status = String::from_utf8(git::run(self.root, &["status", "--short"])?)?;
        let merge = self.git_operation("MERGE_HEAD")?;
        let rebase = self.git_operation("rebase-merge")? || self.git_operation("rebase-apply")?;
        let operation = match (rebase, merge) {
            (true, _) => Some(Operation::Rebase),
            (false, true) => Some(Operation::Merge),
            (false, false) => None,
        };
        Ok(Observation {
            branch: branch.trim().into(),
            revision: self.head()?.unwrap_or_default(),
            status: status.trim().into(),
            operation,
        })
    }

    fn git_operation(&self, name: &str) -> Result<bool> {
        let path = String::from_utf8(git::run(self.root, &["rev-parse", "--git-path", name])?)?;
        Ok(self.root.join(path.trim()).exists())
    }
}
