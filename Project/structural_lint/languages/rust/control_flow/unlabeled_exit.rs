use crate::structural_lint::languages::measure::{Code, Measurement, justified};
use crate::structural_lint::languages::rust::context::in_test_context;
use crate::structural_lint::rule_identities::control_flow::ControlFlow;
use crate::structural_lint::rule_identities::kind::Kind;
use tree_sitter::Node;

const LOOPS: &[&str] = &["loop_expression", "while_expression", "for_expression"];
const BOUNDARIES: &[&str] = &["closure_expression", "function_item", "async_block"];

// DECISION: D155
pub(super) fn inspect(exit: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let mut cursor = exit.walk();
    let labeled = exit
        .named_children(&mut cursor)
        .any(|child| child.kind() == "label");
    let nested =
        !labeled && enclosing_loop(exit).is_some_and(|inner| enclosing_loop(inner).is_some());
    let exempt = !nested || justified(exit, source) || in_test_context(exit, source);
    if exempt {
        return;
    }
    let symbol = match exit.kind() {
        "continue_expression" => "continue",
        _ => "break",
    };
    output.push(Measurement {
        kind: Kind::ControlFlow(ControlFlow::UnlabeledExit),
        line: exit.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn enclosing_loop(node: Node<'_>) -> Option<Node<'_>> {
    let mut current = node.parent()?;
    loop {
        let kind = current.kind();
        let boundary = BOUNDARIES.contains(&kind);
        if boundary {
            return None;
        }
        let found = LOOPS.contains(&kind);
        if found {
            return Some(current);
        }
        current = current.parent()?;
    }
}
