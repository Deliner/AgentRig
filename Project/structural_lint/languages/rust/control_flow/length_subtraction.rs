use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, text};
use crate::structural_lint::languages::rust::exemptions::in_test_context;
use crate::structural_lint::rule_identities::{ControlFlow, Kind};
use tree_sitter::Node;

// DECISION: D155
// DECISION: D210
pub(super) fn inspect(binary: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let subtraction = binary
        .child_by_field_name("operator")
        .is_some_and(|operator| text(operator, source) == "-");
    let Some(method) = subtraction
        .then(|| binary.child_by_field_name("left"))
        .flatten()
        .and_then(|left| counted(left, source))
    else {
        return;
    };
    let allowed = in_test_context(binary, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::ControlFlow(ControlFlow::LengthSubtraction),
        line: binary.start_position().row + 1,
        symbol: method.into(),
        actual: 1,
    });
}
fn counted<'a>(operand: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    length(operand, source).or_else(|| length(sole_argument(operand)?, source))
}
fn sole_argument(call: Node<'_>) -> Option<Node<'_>> {
    let arguments = call.child_by_field_name("arguments")?;
    let single = arguments.named_child_count() == 1;
    single.then(|| arguments.named_child(0)).flatten()
}
fn length<'a>(call: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let nullary = call.kind() == "call_expression"
        && call
            .child_by_field_name("arguments")
            .is_some_and(|arguments| arguments.named_child_count() == 0);
    let field = call
        .child_by_field_name("function")
        .filter(|function| nullary && function.kind() == "field_expression")?
        .child_by_field_name("field")?;
    let name = text(field, source);
    matches!(name, "len" | "count" | "count_ones" | "count_zeros").then_some(name)
}
