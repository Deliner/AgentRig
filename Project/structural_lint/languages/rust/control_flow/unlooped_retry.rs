use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::exemptions::async_code::scope_boundary;
use crate::structural_lint::languages::rust::exemptions::{in_test_context, last_segment};
use crate::structural_lint::rule_identities::control_flow::ControlFlow;
use crate::structural_lint::rule_identities::kind::Kind;
use std::iter::successors;
use tree_sitter::Node;

pub(super) fn inspect(call: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(symbol) = retried(call, source) else {
        return;
    };
    let looped = successors(call.parent(), Node::parent)
        .take_while(|node| !scope_boundary(*node))
        .any(|node| {
            matches!(
                node.kind(),
                "loop_expression" | "while_expression" | "for_expression"
            )
        });
    let allowed = looped || justified(call, source) || in_test_context(call, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::ControlFlow(ControlFlow::UnloopedRetry),
        line: call.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn retried<'a>(call: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let function = call.child_by_field_name("function")?;
    match function.kind() {
        "field_expression" => {
            let name = text(function.child_by_field_name("field")?, source);
            (name == "compare_exchange_weak").then_some(name)
        }
        "scoped_identifier" => {
            let name = last_segment(function, source);
            let threaded = function
                .child_by_field_name("path")
                .is_some_and(|path| last_segment(path, source) == "thread");
            let parked = threaded && matches!(name, "park" | "park_timeout");
            parked.then_some(name)
        }
        _ => None,
    }
}
