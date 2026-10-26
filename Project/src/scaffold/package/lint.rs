// DECISION: D021
// DECISION: D036
use crate::lint::rules;
use anyhow::Result;
use serde_json::json;

pub fn template(settings: &super::Config) -> Result<String> {
    let skills = &settings.paths.skills;
    let sources = &settings.paths.sources;
    let rules: Vec<_> = rules::ALL
        .iter()
        .filter(|kind| {
            !matches!(
                kind,
                rules::Kind::DirectoryArchitecture
                    | rules::Kind::LineWidth
                    | rules::Kind::ForbiddenFile
            )
        })
        .map(|kind| rules::example(*kind, skills, sources))
        .collect();
    let config = json!({
        "version": 1,
        "config_skill": format!("{skills}/repair/SKILL.md"),
        "exclude": [".git/**", settings.paths.service_path("bin/**"), format!("{}/**", settings.paths.runtime), "**/target/**", "**/__pycache__/**", "**/.pytest_cache/**"],
        "rules": rules,
    });
    crate::yaml::encode(&config)
}
