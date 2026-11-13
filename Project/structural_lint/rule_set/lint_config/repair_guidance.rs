use super::{Config, Level, Rule};
use std::path::Path;

impl Config {
    pub fn resolve_guidance(&mut self, resources: &Path) {
        self.config_skill = resources
            .join(&self.config_skill)
            .to_string_lossy()
            .into_owned();
        for rule in &mut self.rules {
            rule.warning_skill = resources
                .join(&rule.warning_skill)
                .to_string_lossy()
                .into_owned();
            rule.error_skill = resources
                .join(&rule.error_skill)
                .to_string_lossy()
                .into_owned();
        }
    }
}
impl Rule {
    pub fn skill(&self, level: Level) -> String {
        match level {
            Level::Warning => self.warning_skill.clone(),
            Level::Error => self.error_skill.clone(),
        }
    }
}
