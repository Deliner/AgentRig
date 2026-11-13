use crate::structural_lint::languages::neutral_metrics::{Code, text};
use crate::structural_lint::languages::rust::exemptions::last_segment;
use tree_sitter::Node;

pub(super) fn inspect(comparison: Node<'_>, source: Code<'_>) -> Option<&'static str> {
    let operator = comparison.child_by_field_name("operator")?.kind();
    let compared = matches!(operator, "<" | "<=" | ">" | ">=" | "==");
    let operands = [
        comparison.child_by_field_name("left")?,
        comparison.child_by_field_name("right")?,
    ];
    let tolerance = compared && operands.into_iter().any(|operand| epsilon(operand, source));
    tolerance.then_some("EPSILON")
}
fn epsilon(operand: Node<'_>, source: Code<'_>) -> bool {
    let mut current = bare(operand);
    while let Some(inner) = current.named_child(0).filter(|_| negated(current)) {
        current = bare(inner);
    }
    let scoped = current.kind() == "scoped_identifier";
    let named = current
        .child_by_field_name("name")
        .is_some_and(|name| text(name, source) == "EPSILON");
    let floating = current
        .child_by_field_name("path")
        .is_some_and(|path| matches!(last_segment(path, source), "f32" | "f64"));
    scoped && named && floating
}
fn negated(operand: Node<'_>) -> bool {
    operand.kind() == "unary_expression" && operand.child(0).is_some_and(|sign| sign.kind() == "-")
}
fn bare(mut operand: Node<'_>) -> Node<'_> {
    while let Some(inner) = operand
        .named_child(0)
        .filter(|_| operand.kind() == "parenthesized_expression")
    {
        operand = inner;
    }
    operand
}
