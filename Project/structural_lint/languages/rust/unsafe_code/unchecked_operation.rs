use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::exemptions::{in_test_context, last_segment};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use std::iter::successors;
use tree_sitter::Node;

pub(super) fn inspect(call: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some((reference, name)) = operation(call, source) else {
        return;
    };
    let allowed = justified(call, source) || in_test_context(call, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(reference),
        line: call.start_position().row + 1,
        symbol: name.into(),
        actual: 1,
    });
}
fn operation<'a>(call: Node<'_>, source: Code<'a>) -> Option<(Reference, &'a str)> {
    let function = call.child_by_field_name("function")?;
    let target = function
        .child_by_field_name("function")
        .filter(|_| function.kind() == "generic_function")
        .unwrap_or(function);
    let (name, path) = match target.kind() {
        "identifier" | "scoped_identifier" => (last_segment(target, source), true),
        "field_expression" => (text(target.child_by_field_name("field")?, source), false),
        _ => return None,
    };
    let transmuted = path && matches!(name, "transmute" | "transmute_copy");
    let raised = unchecked(name) && !constant(call) && !inherited(call, source);
    match (transmuted, raised) {
        (true, _) => Some((Reference::TransmuteCall, name)),
        (false, true) => Some((Reference::UncheckedCall, name)),
        (false, false) => None,
    }
}
fn unchecked(name: &str) -> bool {
    name.ends_with("_unchecked") || name.starts_with("unchecked_")
}
fn constant(call: Node<'_>) -> bool {
    let Some(arguments) = call.child_by_field_name("arguments") else {
        return false;
    };
    let values: Vec<_> = arguments
        .named_children(&mut arguments.walk())
        .filter(|argument| !argument.is_extra())
        .collect();
    let literal = values
        .iter()
        .all(|value| value.kind().ends_with("_literal"));
    !values.is_empty() && literal
}
fn inherited(call: Node<'_>, source: Code<'_>) -> bool {
    successors(call.parent(), Node::parent)
        .find(|node| node.kind() == "function_item")
        .and_then(|function| function.child_by_field_name("name"))
        .is_some_and(|name| unchecked(text(name, source)))
}
