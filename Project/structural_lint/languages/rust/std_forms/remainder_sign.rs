use super::bare;
use crate::structural_lint::languages::neutral_metrics::Code;
use crate::structural_lint::languages::rust::exemptions::zero_literal;
use tree_sitter::Node;

pub(super) fn inspect(comparison: Node<'_>, source: Code<'_>) -> Option<&'static str> {
    let operator = comparison.child_by_field_name("operator")?.kind();
    let equality = matches!(operator, "==" | "!=");
    let ordering = matches!(operator, "<" | "<=" | ">" | ">=");
    let left = comparison.child_by_field_name("left")?;
    let right = comparison.child_by_field_name("right")?;
    let (_, bound) = [(left, right), (right, left)]
        .into_iter()
        .find(|(operand, _)| remainder(bare(*operand)))?;
    let numeric = matches!(bound.kind(), "integer_literal" | "float_literal");
    let signed = equality && !zero_literal(bound, source);
    let flagged = ordering || signed;
    let compared = numeric && flagged;
    compared.then_some("%")
}
fn remainder(operand: Node<'_>) -> bool {
    operand.kind() == "binary_expression"
        && operand
            .child_by_field_name("operator")
            .is_some_and(|operator| operator.kind() == "%")
}
