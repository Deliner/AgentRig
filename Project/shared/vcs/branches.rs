use super::git;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub base: String,
    pub prefix: String,
    // DECISION: D179
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
    // DECISION: D213
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin: Option<String>,
    // DECISION: D218
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install: Option<String>,
    // DECISION: D233
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keep: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            base: "main".into(),
            prefix: "feature/".into(),
            remote: None,
            pin: None,
            install: None,
            keep: Vec::new(),
        }
    }
}

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
        ensure!(
            self.remote.as_deref().is_none_or(git::branch_name),
            "vcs.remote must name a git remote"
        );
        Ok(())
    }
}
