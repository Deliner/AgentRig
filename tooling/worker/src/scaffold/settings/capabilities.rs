use super::{Check, CheckKind};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    #[serde(default = "enabled")]
    pub lint: bool,
}
fn enabled() -> bool {
    true
}
impl Default for Capabilities {
    fn default() -> Self {
        Self { lint: true }
    }
}
impl Capabilities {
    pub(super) fn validate(&self, checks: &[Check]) -> Result<()> {
        ensure!(
            self.lint || !checks.iter().any(|check| check.kind == CheckKind::Lint),
            "capabilities.lint is false but a lint check is configured"
        );
        Ok(())
    }
}
