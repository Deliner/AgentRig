// DECISION: D021
// DECISION: D036
use super::rust::cargo::Layout;
use super::template::{CACHE, SOURCE};
use crate::project_config::Paths;
use crate::structural_lint::rule_identities::{
    Kind, contract_settings::RUST_SEED, structure::Structure,
};
use crate::structural_lint::rule_set::capabilities;
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::BTreeSet;

// DECISION: D189
const FILE_EXTENSIONS: [&str; 15] = [
    ".py", ".sh", ".rs", ".go", ".c", ".h", ".cpp", ".cs", ".java", ".json", ".md", ".toml",
    ".txt", ".yaml", ".yml",
];
// DECISION: D029
// DECISION: D030
// DECISION: D048
// DECISION: D056
const MEMORY_REGISTRIES: [&str; 6] = [
    "Decisions",
    "Invariants",
    "Plan",
    "Backlog",
    "Archive/Plan",
    "Archive/Backlog",
];

pub fn template(settings: &super::Config, rust: &Layout, language: &str) -> Result<String> {
    let paths = &settings.paths;
    let mut rules: Vec<Value> = capabilities::ALL
        .iter()
        .filter(|kind| cataloged(**kind))
        .map(|kind| example(*kind, paths))
        .collect();
    rules.push(architecture(&paths.skills, rust));
    rules.push(program_language(paths, language));
    rules.extend(markdown(paths));
    let skipped = [".git", CACHE, "**/__pycache__", "**/.venv", "**/target"];
    let exclude: Vec<String> = skipped
        .into_iter()
        .chain([paths.skills.as_str()])
        .flat_map(|path| [path.to_owned(), format!("{path}/**")])
        .collect();
    let config = json!({
        "version": 1,
        "config_skill": format!("{}/repair/SKILL.md", paths.skills),
        "exclude": exclude,
        "rules": rules,
    });
    crate::shared::strict_yaml::encode(&config)
}

fn tree() -> Value {
    json!([SOURCE, format!("{SOURCE}/**")])
}

// DECISION: D182
// DECISION: D184
fn example(kind: Kind, paths: &Paths) -> Value {
    let mut rule = capabilities::example(kind, &paths.skills, &["**".into()]);
    match kind {
        // DECISION: D116
        // DECISION: D172
        Kind::Structure(Structure::NonblankLines) => {
            let owned = [paths.lint.clone(), format!("{}/Assets/**", paths.memory)];
            let defaults = rule["exclude"].as_array().cloned().unwrap_or_default();
            let exclude: Vec<Value> = owned.map(Value::from).into_iter().chain(defaults).collect();
            rule["exclude"] = json!(exclude);
            rule["extensions"] = json!(FILE_EXTENSIONS);
        }
        Kind::Structure(Structure::DirectoryEntries) => {
            let registries = MEMORY_REGISTRIES.map(|name| format!("{}/{name}", paths.memory));
            rule["exclude"] = json!(registries);
        }
        Kind::Structure(Structure::DumpName) => rule["include"] = tree(),
        Kind::Structure(_)
        | Kind::ControlFlow(_)
        | Kind::Signature(_)
        | Kind::Ownership(_)
        | Kind::Reference(_) => {}
    }
    rule
}

// DECISION: D195
fn cataloged(kind: Kind) -> bool {
    match kind {
        Kind::Structure(
            Structure::DirectoryArchitecture
            | Structure::LineWidth
            | Structure::ForbiddenFile
            | Structure::ProgramLanguage,
        ) => false,
        Kind::Structure(
            Structure::NonblankLines
            | Structure::SameText
            | Structure::DirectoryEntries
            | Structure::DumpName
            | Structure::FunctionLines
            | Structure::EmbeddedText
            | Structure::TestPlacement,
        )
        | Kind::ControlFlow(_)
        | Kind::Signature(_)
        | Kind::Ownership(_)
        | Kind::Reference(_) => true,
    }
}

// DECISION: D198
fn program_language(paths: &Paths, language: &str) -> Value {
    let kind = Kind::Structure(Structure::ProgramLanguage);
    let mut rule = capabilities::example(kind, &paths.skills, &["**".into()]);
    rule["allowed_languages"] = json!([language]);
    let adapters = [paths.service_path("hooks/**"), paths.service_path("bin/**")];
    rule["exclude"] = json!(adapters);
    rule
}

// DECISION: D192
fn markdown(paths: &Paths) -> Vec<Value> {
    let condense = format!("{skills}/condense-markdown/SKILL.md", skills = paths.skills);
    let edit_state = format!("{skills}/edit-state/SKILL.md", skills = paths.skills);
    let state = format!("{memory}/State.md", memory = paths.memory);
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
        json!({"id": "state-size", "kind": "nonblank-lines", "target": "file",
            "include": [state], "error": 100,
            "warning_skill": edit_state, "error_skill": edit_state}),
        json!({"id": "state-width", "kind": "line-width", "target": "file",
            "include": [state], "error": 120,
            "warning_skill": edit_state, "error_skill": edit_state}),
    ]
}

fn architecture(skills: &str, rust: &Layout) -> Value {
    let kind = Kind::Structure(Structure::DirectoryArchitecture);
    let mut rule = capabilities::example(kind, skills, &[]);
    rule["include"] = tree();
    rule["extensions"] = json!([".rs", ".py", ".pyi"]);
    rule["architecture"]["rust_roots"] = json!(rust.roots);
    rule["architecture"]["feature_root"] = json!(SOURCE);
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
