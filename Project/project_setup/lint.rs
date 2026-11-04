// DECISION: D021
// DECISION: D036
use crate::structural_lint::{catalog, rules};
use anyhow::Result;
use serde_json::{Value, json};

pub fn template(settings: &super::Config) -> Result<String> {
    let skills = &settings.paths.skills;
    let sources = &settings.paths.sources;
    let mut rules: Vec<Value> = catalog::ALL
        .iter()
        .filter(|kind| {
            !matches!(
                kind,
                rules::Kind::DirectoryArchitecture
                    | rules::Kind::LineWidth
                    | rules::Kind::ForbiddenFile
            )
        })
        .map(|kind| catalog::example(*kind, skills, sources))
        .collect();
    rules.push(architecture(skills, sources));
    rules.extend(markdown(skills, &settings.paths.memory, sources));
    let config = json!({
        "version": 1,
        "config_skill": format!("{skills}/repair/SKILL.md"),
        "exclude": [".git/**", settings.paths.service_path("bin/**"), format!("{}/**", settings.paths.runtime), "**/target/**", "**/__pycache__/**", "**/.pytest_cache/**"],
        "rules": rules,
    });
    crate::shared::yaml::encode(&config)
}

fn markdown(skills: &str, memory: &str, sources: &[String]) -> Vec<Value> {
    let condense = format!("{skills}/condense-markdown/SKILL.md");
    let memory_exclude = format!("{memory}/**");
    vec![
        json!({"id": "markdown-size", "kind": "nonblank-lines", "target": "file",
            "include": ["**/README.md", "**/AGENTS.md"], "exclude": [memory_exclude],
            "error": 70, "warning_skill": condense, "error_skill": condense}),
        json!({"id": "markdown-width", "kind": "line-width", "target": "file",
            "include": ["**/README.md", "**/AGENTS.md"], "exclude": [memory_exclude],
            "error": 120, "warning_skill": condense, "error_skill": condense}),
        json!({"id": "markdown-placement", "kind": "forbidden-file", "target": "file",
            "include": sources, "extensions": [".md"],
            "exclude": ["**/README.md", "**/AGENTS.md", "**/SKILL.md"],
            "level": "error", "warning_skill": condense, "error_skill": condense}),
    ]
}

fn architecture(skills: &str, sources: &[String]) -> Value {
    let mut rule = catalog::example(rules::Kind::DirectoryArchitecture, skills, sources);
    let root = sources
        .first()
        .map(|source| source.strip_suffix("/**").unwrap_or(source))
        .unwrap_or(".");
    rule["extensions"] = json!([".rs", ".py", ".pyi"]);
    rule["architecture"]["rust_roots"] = json!([format!("{root}/src/lib.rs")]);
    rule
}
