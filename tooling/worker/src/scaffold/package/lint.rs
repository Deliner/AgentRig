// DECISION: D021
use crate::lint::rules;
use anyhow::Result;
use serde_json::json;

pub fn template(skills: &str, sources: &[String]) -> Result<String> {
    let rules: Vec<_> = rules::ALL
        .iter()
        .map(|kind| rules::example(*kind, skills, sources))
        .collect();
    let config = json!({
        "version": 1,
        "config_skill": format!("{skills}/repair/SKILL.md"),
        "exclude": [".git/**", ".worker/bin/**", ".worker/runtime/**", "**/target/**", "**/__pycache__/**", "**/.pytest_cache/**"],
        "rules": rules,
    });
    review_runner::config::yaml::encode(&config)
}
