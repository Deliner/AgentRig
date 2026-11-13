use super::language_support::{extensions, syntax};
use super::{ALL, Kind};
use crate::structural_lint::languages;
use crate::structural_lint::rule_identities::contract_settings;
use crate::structural_lint::rule_identities::control_flow::ControlFlow;
use crate::structural_lint::rule_identities::ownership::{self, Ownership};
use crate::structural_lint::rule_identities::reference::Reference;
use crate::structural_lint::rule_identities::structure::Structure;
use serde_json::{Value, json};

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
    describe_settings(kind, &mut value);
    value
}
// DECISION: D156
fn describe_settings(kind: Kind, value: &mut Value) {
    match kind {
        Kind::Structure(Structure::DirectoryArchitecture) => {
            value["architecture"] = contract_settings::Settings::describe();
        }
        Kind::Ownership(Ownership::PairedCalls) => {
            value["pairs"] = json!(
                "required nonempty list of open and close method names; a call of open, then an early exit, then a call of close on the token-equal receiver in one block is a finding"
            );
        }
        Kind::ControlFlow(ControlFlow::BooleanExpression) => {
            value["junctions"] = json!({"values": ["mixed", "single"], "default": "mixed", "single": "every expression joining && with || or and with or, directly or inside parentheses, is also an error finding"});
        }
        Kind::Structure(_)
        | Kind::ControlFlow(_)
        | Kind::Signature(_)
        | Kind::Ownership(_)
        | Kind::Reference(_) => {}
    }
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
    match kind {
        Kind::Structure(Structure::DirectoryArchitecture) => {
            value["architecture"] = json!({"python_root": ".", "rust_roots": ["src/lib.rs"]});
        }
        // DECISION: D156
        Kind::Ownership(Ownership::PairedCalls) => value["pairs"] = ownership::default_pairs(),
        // DECISION: D156
        Kind::ControlFlow(ControlFlow::BooleanExpression) => value["junctions"] = json!("single"),
        // DECISION: D182
        Kind::Reference(Reference::CargoRustflags) => {
            value["include"] = json!(["**/.cargo/config.toml"]);
        }
        Kind::Structure(_)
        | Kind::ControlFlow(_)
        | Kind::Signature(_)
        | Kind::Ownership(_)
        | Kind::Reference(_) => {}
    }
    value
}
