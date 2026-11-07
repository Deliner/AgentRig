use super::languages;
pub use super::rules::kind::Kind;
pub use super::rules::{Descriptor, Parameters};
use crate::structural_lint::rules::control_flow::ControlFlow;
use crate::structural_lint::rules::ownership::Ownership;
use crate::structural_lint::rules::reference::Reference;
use crate::structural_lint::rules::signature::Signature;
use crate::structural_lint::rules::structure::Structure;
use anyhow::{Result, bail};
use serde::{Deserialize, Deserializer, de};
use serde_json::{Value, json};
use std::path::Path;

// DECISION: D133
// DECISION: D134
impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        family(&name).ok_or_else(|| {
            let names: Vec<String> = ALL.iter().map(|kind| format!("`{kind}`")).collect();
            de::Error::custom(format!(
                "unknown variant `{name}`, expected one of {}",
                names.join(", ")
            ))
        })
    }
}
fn family(name: &str) -> Option<Kind> {
    let value = || Value::from(name);
    serde_json::from_value(value())
        .map(Kind::Structure)
        .or_else(|_| serde_json::from_value(value()).map(Kind::ControlFlow))
        .or_else(|_| serde_json::from_value(value()).map(Kind::Signature))
        .or_else(|_| serde_json::from_value(value()).map(Kind::Ownership))
        .or_else(|_| serde_json::from_value(value()).map(Kind::Reference))
        .ok()
}

// DECISION: D016
// DECISION: D017
// DECISION: D018
pub const ALL: &[Kind] = &[
    Kind::Structure(Structure::NonblankLines),
    Kind::Structure(Structure::LineWidth),
    Kind::Structure(Structure::ForbiddenFile),
    Kind::Structure(Structure::DirectoryEntries),
    Kind::Structure(Structure::DumpName),
    Kind::Structure(Structure::FunctionLines),
    Kind::Signature(Signature::ParameterCount),
    Kind::ControlFlow(ControlFlow::NestingDepth),
    Kind::Signature(Signature::BooleanParameter),
    Kind::ControlFlow(ControlFlow::BranchBudget),
    Kind::Reference(Reference::UnsafeJustification),
    Kind::Reference(Reference::HardcodedSecret),
    Kind::Reference(Reference::SilentError),
    Kind::Reference(Reference::DisabledTest),
    Kind::Reference(Reference::AllowJustification),
    Kind::Reference(Reference::DebugArtifact),
    Kind::Reference(Reference::DebugAssertion),
    Kind::Reference(Reference::PanicPath),
    Kind::Reference(Reference::WildcardImport),
    Kind::Signature(Signature::TupleAsRecord),
    Kind::Reference(Reference::EagerFallback),
    Kind::Reference(Reference::DiscardedResult),
    Kind::Ownership(Ownership::DisabledRaii),
    Kind::Ownership(Ownership::GlobalState),
    Kind::Reference(Reference::SuperSuperReach),
    Kind::Ownership(Ownership::RecursiveDeletion),
    Kind::Reference(Reference::UnwindJustification),
    Kind::Ownership(Ownership::GuardAcrossAwait),
    Kind::Reference(Reference::BlockingInAsync),
    Kind::Signature(Signature::OrdEqMismatch),
    Kind::Signature(Signature::SecretDebug),
    Kind::ControlFlow(ControlFlow::BooleanExpression),
    Kind::Structure(Structure::EmbeddedText),
    Kind::ControlFlow(ControlFlow::NamedIfCondition),
    Kind::Reference(Reference::CommentReference),
    Kind::Structure(Structure::DirectoryArchitecture),
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
    let architecture = kind == Kind::Structure(Structure::DirectoryArchitecture);
    if architecture {
        value["architecture"] = super::rules::architecture::Settings::describe();
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
    let excludes = !descriptor.exclude.is_empty();
    if excludes {
        value["exclude"] = json!(descriptor.exclude);
    }
    let language_rule = syntax(kind);
    if language_rule {
        value["extensions"] = json!(extensions(kind));
    }
    let architecture = kind == Kind::Structure(Structure::DirectoryArchitecture);
    if architecture {
        value["architecture"] = json!({"python_root": ".", "rust_roots": ["src/lib.rs"]});
    }
    value
}
