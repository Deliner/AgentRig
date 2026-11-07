// DECISION: D021
// DECISION: D036
use super::cargo::Layout;
use crate::structural_lint::rules::{architecture::RUST_SEED, structure::Structure};
use crate::structural_lint::{catalog, rules};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub fn template(settings: &super::Config, rust: &Layout) -> Result<String> {
    let skills = &settings.paths.skills;
    let sources = &settings.paths.sources;
    let mut rules: Vec<Value> = catalog::ALL
        .iter()
        .filter(|kind| {
            !matches!(
                kind,
                rules::Kind::Structure(Structure::DirectoryArchitecture)
                    | rules::Kind::Structure(Structure::LineWidth)
                    | rules::Kind::Structure(Structure::ForbiddenFile)
            )
        })
        .map(|kind| catalog::example(*kind, skills, sources))
        .collect();
    rules.push(architecture(skills, sources, rust));
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

fn architecture(skills: &str, sources: &[String], rust: &Layout) -> Value {
    let mut rule = catalog::example(
        rules::Kind::Structure(Structure::DirectoryArchitecture),
        skills,
        sources,
    );
    rule["extensions"] = json!([".rs", ".py", ".pyi"]);
    rule["architecture"]["rust_roots"] = json!(rust.roots);
    let crates = !rust.crates.is_empty();
    if crates {
        rule["architecture"]["rust_crates"] = json!(rust.crates);
    }
    // DECISION: D136
    let externals = !rust.externals.is_empty();
    if externals {
        let seed = RUST_SEED.map(str::to_owned);
        let names: BTreeSet<_> = seed.into_iter().chain(rust.externals.clone()).collect();
        rule["architecture"]["external"] = json!({"rust": names});
    }
    rule
}
