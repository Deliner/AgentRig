use super::{Check, CheckKind, relative};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    #[serde(default = "enabled")]
    pub lint: bool,
    pub review: Option<Resource>,
    pub delegation: Option<Resource>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub config: String,
}
fn enabled() -> bool {
    true
}
impl Default for Capabilities {
    fn default() -> Self {
        Self {
            lint: true,
            review: None,
            delegation: None,
        }
    }
}
impl Capabilities {
    pub(super) fn validate(&self, root: &Path, checks: &[Check]) -> Result<()> {
        ensure!(
            self.lint || !checks.iter().any(|check| check.kind == CheckKind::Lint),
            "capabilities.lint is false but a lint check is configured"
        );
        if let Some(review) = &self.review {
            let path = relative(root, &review.config).context("capabilities.review.config")?;
            review_runner::config::load(&path).context("capabilities.review.config")?;
        }
        if let Some(delegation) = &self.delegation {
            let path =
                relative(root, &delegation.config).context("capabilities.delegation.config")?;
            agentrig::delegate::config::load(&path).context("capabilities.delegation.config")?;
        }
        Ok(())
    }
}
