use super::{Settings, git};
use anyhow::{Result, ensure};

impl Settings {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            git::branch_name(&self.base),
            "vcs.base must name a valid git branch without normalization"
        );
        ensure!(
            !self.prefix.is_empty()
                && git::branch_name(&format!("{}example", self.prefix))
                && !self.base.starts_with(&self.prefix),
            "vcs.prefix must form valid git branches distinct from vcs.base"
        );
        Ok(())
    }
}
