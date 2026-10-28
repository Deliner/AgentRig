// DECISION: D021
// DECISION: D036
use crate::structural_lint::rules;
use anyhow::Result;
use serde_json::{Value, json};

pub fn template(settings: &super::Config) -> Result<String> {
    let skills = &settings.paths.skills;
    let sources = &settings.paths.sources;
    let mut rules: Vec<Value> = rules::ALL
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
    rules.push(architecture(skills, sources));
    let config = json!({
        "version": 1,
        "config_skill": format!("{skills}/repair/SKILL.md"),
        "exclude": [".git/**", settings.paths.service_path("bin/**"), format!("{}/**", settings.paths.runtime), "**/target/**", "**/__pycache__/**", "**/.pytest_cache/**"],
        "rules": rules,
    });
    crate::shared::yaml::encode(&config)
}

fn architecture(skills: &str, sources: &[String]) -> Value {
    let mut rule = rules::example(rules::Kind::DirectoryArchitecture, skills, sources);
    let root = sources
        .first()
        .map(|source| source.strip_suffix("/**").unwrap_or(source))
        .unwrap_or(".");
    rule["extensions"] = json!([".rs", ".py", ".pyi"]);
    rule["architecture"]["rust_roots"] = json!([format!("{root}/src/lib.rs")]);
    rule
}
