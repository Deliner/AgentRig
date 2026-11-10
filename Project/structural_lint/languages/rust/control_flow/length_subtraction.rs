use crate::structural_lint::languages::measure::{Measurement, justified, text};
use crate::structural_lint::languages::rust::context::in_test_context;
use crate::structural_lint::rules::control_flow::ControlFlow;
use crate::structural_lint::rules::kind::Kind;
use tree_sitter::Node;

// DECISION: D155
pub(super) fn inspect(binary: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let subtraction = binary
        .child_by_field_name("operator")
        .is_some_and(|operator| text(operator, source) == "-");
    let Some(method) = binary
        .child_by_field_name("left")
        .filter(|_| subtraction)
        .and_then(|left| length(left, source))
    else {
        return;
    };
    let allowed = justified(binary, source) || in_test_context(binary, source);
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
fn length<'a>(call: Node<'_>, source: &'a str) -> Option<&'a str> {
    let nullary = call.kind() == "call_expression"
        && call
            .child_by_field_name("arguments")
            .is_some_and(|arguments| arguments.named_child_count() == 0);
    let field = call
        .child_by_field_name("function")
        .filter(|function| nullary && function.kind() == "field_expression")?
        .child_by_field_name("field")?;
    let name = text(field, source);
    matches!(name, "len" | "count").then_some(name)
}
