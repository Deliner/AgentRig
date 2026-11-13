use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::exemptions::in_test_context;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::ownership::Ownership;
use tree_sitter::Node;
// DECISION: D092
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let Some(symbol) = raii_marker(function, source) else {
        return;
    };
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Ownership(Ownership::DisabledRaii),
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn raii_marker<'a>(function: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    if let Some(field) = function.child_by_field_name("field") {
        let method = field.kind() == "field_identifier";
        let name = text(field, source);
        let consumes = method && matches!(name, "keep" | "into_path");
        return consumes.then_some(name);
    }
    let unscoped = function.kind() != "scoped_identifier";
    if unscoped {
        return None;
    }
    let mut segments = text(function, source).rsplit("::");
    let last = segments.next()?;
    let prior = segments.next()?;
    match (prior, last) {
        ("mem", "forget") => Some("forget"),
        ("ManuallyDrop", "new") => Some("ManuallyDrop"),
        ("Box", "leak") => Some("leak"),
        _ => None,
    }
}
