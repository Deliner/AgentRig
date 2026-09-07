mod descriptor;
pub mod kind;
use super::languages;
use anyhow::{Result, bail};
pub use descriptor::{Descriptor, Parameters};
pub use kind::Kind;
use serde_json::{Value, json};
use std::path::Path;

// DECISION: D016
// DECISION: D017
// DECISION: D018
pub const ALL: &[Kind] = &[
    Kind::NonblankLines,
    Kind::DirectoryEntries,
    Kind::FunctionLines,
    Kind::ParameterCount,
    Kind::NamedIfCondition,
    Kind::DirectoryArchitecture,
];
pub fn syntax(kind: Kind) -> bool {
    languages::HANDLERS
        .iter()
        .any(|handler| handler.rules.contains(&kind))
}
pub fn supports_path(kind: Kind, path: &Path) -> bool {
    languages::handler(path).is_some_and(|handler| handler.rules.contains(&kind))
}
pub fn extensions(kind: Kind) -> Vec<&'static str> {
    languages::HANDLERS
        .iter()
        .filter(|handler| handler.rules.contains(&kind))
        .flat_map(|handler| handler.extensions.iter().copied())
        .collect()
}
pub fn support(kind: Kind) -> String {
    languages::HANDLERS
        .iter()
        .filter(|handler| handler.rules.contains(&kind))
        .map(|handler| format!("{} ({})", handler.title, handler.extensions.join(", ")))
        .collect::<Vec<_>>()
        .join(" and ")
}
pub fn validate_includes(kind: Kind, patterns: &[String]) -> Result<()> {
    let structural = !syntax(kind) || kind.descriptor().target == "directory";
    if structural {
        return Ok(());
    }
    for pattern in patterns {
        if let Some(ext) = Path::new(pattern).extension().and_then(|ext| ext.to_str()) {
            let literal = !ext.contains(['*', '?', '[', ']', '{', '}', '\\']);
            if literal {
                validate_extensions(kind, &[format!(".{ext}")])?;
            }
        }
    }
    Ok(())
}
pub fn validate_extensions(kind: Kind, selected: &[String]) -> Result<()> {
    let supported = extensions(kind);
    let unsupported = syntax(kind)
        && selected
            .iter()
            .any(|ext| !supported.contains(&ext.as_str()));
    if unsupported {
        bail!(
            "{kind}: requested extensions {selected:?} are incompatible; handlers support {}",
            support(kind)
        );
    }
    Ok(())
}
pub fn catalog() -> Value {
    Value::Array(ALL.iter().map(|kind| describe(*kind)).collect())
}
pub fn describe(kind: Kind) -> Value {
    let descriptor = kind.descriptor();
    let mut value = json!({"kind": kind, "target": descriptor.target,
        "metric": descriptor.metric, "skill": descriptor.skill,
        "parameters": descriptor.parameters.describe(), "defaults": example(kind, ".agents/skills", &["src/**".into()]),
        "languages": "any UTF-8 text", "extensions": "configurable"});
    let directory = descriptor.target == "directory";
    if directory {
        value["languages"] = json!("any");
        value["extensions"] = json!("not applicable");
    }
    let language_rule = syntax(kind);
    if language_rule {
        let handlers: std::collections::BTreeMap<_, _> = languages::HANDLERS
            .iter()
            .filter(|handler| handler.rules.contains(&kind))
            .map(|handler| (handler.name, handler.extensions))
            .collect();
        value["languages"] = json!(handlers.keys().collect::<Vec<_>>());
        value["handlers"] = json!(handlers);
        value["extensions"] = json!(extensions(kind));
    }
    let architecture = kind == Kind::DirectoryArchitecture;
    if architecture {
        value["architecture"] = super::architecture::Settings::describe();
    }
    value
}
pub fn example(kind: Kind, skills: &str, sources: &[String]) -> Value {
    let descriptor = kind.descriptor();
    let skill = format!("{skills}/{}/SKILL.md", descriptor.skill);
    let mut value = json!({"id": descriptor.default_id, "kind": kind,
        "target": descriptor.target, "include": sources,
        "warning_skill": skill, "error_skill": skill});
    descriptor.parameters.apply(&mut value);
    let language_rule = syntax(kind);
    if language_rule {
        value["extensions"] = json!(extensions(kind));
    }
    let architecture = kind == Kind::DirectoryArchitecture;
    if architecture {
        value["architecture"] = json!({"python_root": ".", "rust_roots": ["src/lib.rs"]});
    }
    value
}
