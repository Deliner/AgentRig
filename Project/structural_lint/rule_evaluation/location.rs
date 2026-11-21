use crate::structural_lint::rule_set::lint_config::Config;
use anyhow::Result;
use std::path::Path;

// DECISION: D016
// DECISION: D017
// DECISION: D018
#[derive(Clone, Copy)]
pub struct Location<'a> {
    pub root: &'a Path,
    pub config: &'a Path,
}
impl Location<'_> {
    pub fn load(self) -> Result<Config> {
        let mut config: Config = crate::shared::strict_yaml::read(self.config)?;
        let directory = self.config.parent().unwrap_or(self.root);
        let external = config
            .skill_root
            .as_ref()
            .map(|skills| directory.join(skills).canonicalize())
            .transpose()?;
        config.validate(external.as_deref().unwrap_or(self.root))?;
        if let Some(resources) = external {
            config.resolve_guidance(&resources);
        }
        Ok(config)
    }
}
