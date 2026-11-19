use crate::shared::calling_agent::{Agent, Caller};
use serde_json::Value;
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
pub const UNNAMED_SKILL: &str = "the repair skill named by config_skill in .agentrig/project.yaml, by default .agentrig/skills/repair/SKILL.md";
fn repair_skill<'a>(config: &'a Value, command: &str) -> Option<&'a str> {
    let memory = command == "memory-check";
    let check_skill = config
        .get("checks")
        .and_then(Value::as_array)
        .and_then(|checks| {
            checks
                .iter()
                .find(|check| memory && check.get("kind").and_then(Value::as_str) == Some("memory"))
        })
        .and_then(|check| check.get("skill"));
    check_skill
        .or_else(|| config.get("config_skill"))
        .and_then(Value::as_str)
}
pub(super) fn configured(root: &Path) -> Option<Value> {
    crate::shared::strict_yaml::read::<Value>(&root.join(".agentrig/project.yaml")).ok()
}
fn through(root: &Path, config: Option<&Value>, agent: Option<Agent>) -> Caller {
    let skills = config
        .and_then(|config| config.pointer("/paths/skills"))
        .and_then(Value::as_str);
    match skills {
        Some(skills) => Caller {
            root: root.to_owned(),
            skills: skills.into(),
            agent,
        },
        None => Caller::plain(root),
    }
}
// DECISION: D188
pub fn caller(root: &Path, agent: Option<Agent>) -> Caller {
    through(root, configured(root).as_ref(), agent)
}
pub(super) fn repair(
    root: &Path,
    config: &Value,
    command: &str,
    agent: Option<Agent>,
) -> Option<String> {
    let skill = repair_skill(config, command)?;
    Some(through(root, Some(config), agent).skill(skill))
}
// DECISION: D188
pub fn config_skill(root: &Path, agent: Option<Agent>) -> Option<String> {
    repair(root, &configured(root)?, "config-check", agent)
}
