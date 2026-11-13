// DECISION: D021
// DECISION: D036
use super::rust::cargo::Layout;
use crate::project_config::Paths;
use crate::structural_lint::rule_identities;
use crate::structural_lint::rule_identities::{
    contract_settings::RUST_SEED, reference::Reference, structure::Structure,
};
use crate::structural_lint::rule_set::capabilities;
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub fn template(settings: &super::Config, rust: &Layout) -> Result<String> {
    let skills = &settings.paths.skills;
    let sources = &settings.paths.sources;
    let mut rules: Vec<Value> = capabilities::ALL
        .iter()
        .filter(|kind| cataloged(**kind))
        .map(|kind| example(*kind, skills, sources))
        .collect();
    rules.push(architecture(skills, sources, rust));
    rules.extend(markdown(&settings.paths));
    let config = json!({
        "version": 1,
        "config_skill": format!("{skills}/repair/SKILL.md"),
        "exclude": [".git/**", settings.paths.service_path("bin/**"), format!("{}/**", settings.paths.runtime), "**/target/**", "**/__pycache__/**", "**/.pytest_cache/**"],
        "rules": rules,
    });
    crate::shared::strict_yaml::encode(&config)
}

fn example(kind: rule_identities::Kind, skills: &str, sources: &[String]) -> Value {
    let mut rule = capabilities::example(kind, skills, sources);
    // DECISION: D182
    let flags = matches!(
        kind,
        rule_identities::Kind::Reference(Reference::AllowJustification)
    );
    if flags {
        let include: Vec<String> = sources
            .iter()
            .cloned()
            .chain([crate::project_config::module("rust")])
            .collect();
        rule["include"] = json!(include);
    }
    rule
}

fn cataloged(kind: rule_identities::Kind) -> bool {
    match kind {
        rule_identities::Kind::Structure(
            Structure::DirectoryArchitecture | Structure::LineWidth | Structure::ForbiddenFile,
        ) => false,
        rule_identities::Kind::Structure(
            Structure::NonblankLines
            | Structure::DirectoryEntries
            | Structure::DumpName
            | Structure::FunctionLines
            | Structure::EmbeddedText
            | Structure::TestPlacement,
        )
        | rule_identities::Kind::ControlFlow(_)
        | rule_identities::Kind::Signature(_)
        | rule_identities::Kind::Ownership(_)
        | rule_identities::Kind::Reference(_) => true,
    }
}

fn markdown(paths: &Paths) -> Vec<Value> {
    let condense = format!("{skills}/condense-markdown/SKILL.md", skills = paths.skills);
    let memory_exclude = format!("{memory}/**", memory = paths.memory);
    let sources = &paths.sources;
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
    let mut rule = capabilities::example(
        rule_identities::Kind::Structure(Structure::DirectoryArchitecture),
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
