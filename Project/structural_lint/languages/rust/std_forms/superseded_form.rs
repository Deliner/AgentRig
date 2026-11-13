use super::{method, sole_argument};
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, justified, text};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use std::iter::successors;
use tree_sitter::Node;

// DECISION: D177
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let form = match node.kind() {
        "call_expression" => trimmed(node, source).or_else(|| filled(node, source)),
        "type_identifier" => (text(node, source) == "Infallible").then_some("Infallible -> !"),
        _ => None,
    };
    let Some(form) = form else {
        return;
    };
    let allowed = justified(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::SupersededForm),
        line: node.start_position().row + 1,
        symbol: form.into(),
        actual: 1,
    });
}
fn trimmed(call: Node<'_>, source: Code<'_>) -> Option<&'static str> {
    let (name, stripped) = method(call, source)?;
    let fallback = name == "unwrap_or";
    let argument = sole_argument(call).filter(|_| fallback)?;
    let (strip, receiver) = method(stripped, source)?;
    let form = match strip {
        "strip_prefix" => "strip_prefix(p).unwrap_or(x) -> trim_prefix(p)",
        "strip_suffix" => "strip_suffix(p).unwrap_or(x) -> trim_suffix(p)",
        _ => return None,
    };
    let borrowed = argument.kind() == "reference_expression";
    let operand = argument
        .child_by_field_name("value")
        .filter(|_| borrowed)
        .unwrap_or(argument);
    let same = text(operand, source) == text(receiver, source);
    same.then_some(form)
}
fn filled(call: Node<'_>, source: Code<'_>) -> Option<&'static str> {
    let function = call.child_by_field_name("function")?;
    let turbofish = function.kind() == "generic_function";
    let (access, target) = if turbofish {
        let arguments = function.child_by_field_name("type_arguments")?;
        (
            function.child_by_field_name("function")?,
            arguments.named_child(0)?,
        )
    } else {
        let bound = call
            .parent()
            .filter(|parent| parent.kind() == "let_declaration")
            .and_then(|binding| binding.child_by_field_name("type"));
        (function, bound.or_else(|| returned(call))?)
    };
    let collected = access
        .child_by_field_name("field")
        .map(|field| text(field, source));
    let vector = target.kind() == "generic_type"
        && target
            .child_by_field_name("type")
            .is_some_and(|name| text(name, source).rsplit("::").next() == Some("Vec"));
    let into_vector = collected == Some("collect") && vector;
    let mapped = access
        .child_by_field_name("value")
        .filter(|_| into_vector)?;
    let (name, range) = method(mapped, source)?;
    let indexed = name == "map" && sole_argument(mapped).is_some() && from_zero(range, source);
    indexed.then_some("(0..n).map(f).collect() -> Vec::from_fn(n, f)")
}
// DECISION: D178
fn returned(call: Node<'_>) -> Option<Node<'_>> {
    let parent = call.parent()?;
    let owner = match parent.kind() {
        "return_expression" => {
            successors(parent.parent(), Node::parent).find(|node| callable(*node))
        }
        "block" => parent
            .parent()
            .filter(|owner| callable(*owner) && owner.child_by_field_name("body") == Some(parent)),
        _ => None,
    };
    owner?.child_by_field_name("return_type")
}
fn callable(node: Node<'_>) -> bool {
    matches!(node.kind(), "function_item" | "closure_expression")
}
fn from_zero(parenthesized: Node<'_>, source: Code<'_>) -> bool {
    let Some(range) = parenthesized
        .named_child(0)
        .filter(|_| parenthesized.kind() == "parenthesized_expression")
        .filter(|inner| inner.kind() == "range_expression" && inner.named_child_count() == 2)
    else {
        return false;
    };
    let exclusive = range
        .child(1)
        .is_some_and(|operator| operator.kind() == "..");
    let zero = range.named_child(0).is_some_and(|start| {
        start.kind() == "integer_literal" && matches!(text(start, source), "0" | "0usize")
    });
    exclusive && zero
}
