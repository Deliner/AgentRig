use super::call_syntax::{method, sole_argument};
use crate::structural_lint::languages::neutral_metrics::Code;
use crate::structural_lint::languages::rust::exemptions::last_segment;
use tree_sitter::Node;

// DECISION: D209
pub(super) fn inspect(arithmetic: Node<'_>, source: Code<'_>) -> Option<&'static str> {
    let operator = arithmetic.child_by_field_name("operator")?.kind();
    let growing = matches!(operator, "+" | "*");
    let operands = [
        arithmetic.child_by_field_name("left")?,
        arithmetic.child_by_field_name("right")?,
    ];
    let bounded = growing && operands.into_iter().any(|operand| maximum(operand, source));
    bounded.then_some("MAX")
}
fn maximum(operand: Node<'_>, source: Code<'_>) -> bool {
    let fallback = method(operand, source).is_some_and(|(name, _)| name == "unwrap_or");
    fallback
        && sole_argument(operand).is_some_and(|argument| last_segment(argument, source) == "MAX")
}
