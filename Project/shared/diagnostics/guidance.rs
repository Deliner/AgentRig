use std::{fmt, path::Path};

pub struct Guidance<'a> {
    pub level: &'a str,
    pub id: &'a str,
    pub location: &'a str,
    pub message: &'a str,
    pub skill: &'a str,
    pub rerun: &'a str,
}
impl fmt::Display for Guidance<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        let advisory = self.level.eq_ignore_ascii_case("warning");
        let action = if advisory { "Consider" } else { "Apply" };
        write!(
            output,
            "{level} [{id}] {location}: {message}. ACTION: {action} {skill}. RERUN: {rerun}",
            level = self.level.to_uppercase(),
            id = self.id,
            location = self.location,
            message = self.message,
            skill = self.skill,
            rerun = self.rerun
        )
    }
}
pub(super) fn repair_skill<'a>(config: Option<&'a serde_json::Value>, command: &str) -> &'a str {
    let memory = command == "memory-check";
    let check_skill = config
        .and_then(|config| config.get("checks"))
        .and_then(serde_json::Value::as_array)
        .and_then(|checks| {
            checks.iter().find(|check| {
                memory && check.get("kind").and_then(serde_json::Value::as_str) == Some("memory")
            })
        })
        .and_then(|check| check.get("skill"));
    check_skill
        .or_else(|| config.and_then(|config| config.get("config_skill")))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("the repair skill named by config_skill in .agentrig/project.yaml, by default .agentrig/skills/repair/SKILL.md")
}
pub fn config_skill(root: &Path) -> String {
    let config =
        crate::shared::strict_yaml::read::<serde_json::Value>(&root.join(".agentrig/project.yaml"))
            .ok();
    repair_skill(config.as_ref(), "config-check").to_owned()
}
