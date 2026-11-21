use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, text};
use crate::structural_lint::languages::rust::exemptions::in_test_context;
use crate::structural_lint::rule_identities::{Kind, Reference};
use tree_sitter::Node;

// DECISION: D089
// DECISION: D168
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(field) = node
        .child_by_field_name("function")
        .and_then(|function| function.child_by_field_name("field"))
    else {
        return;
    };
    let method = field.kind() == "field_identifier";
    let name = text(field, source);
    let eager = method && matches!(name, "unwrap_or" | "ok_or" | "map_or" | "or");
    let unrelated = !eager;
    if unrelated {
        return;
    }
    let safe = !evaluated(node);
    if safe {
        return;
    }
    let allowed = in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::EagerFallback),
        line: node.start_position().row + 1,
        symbol: name.into(),
        actual: 1,
    });
}
fn evaluated(call: Node<'_>) -> bool {
    let Some(arguments) = call.child_by_field_name("arguments") else {
        return false;
    };
    let mut cursor = arguments.walk();
    let Some(first) = arguments
        .named_children(&mut cursor)
        .find(|argument| !argument.is_extra())
    else {
        return false;
    };
    propagates(first) || divides(first)
}
fn propagates(node: Node<'_>) -> bool {
    let shortcut = node.kind() == "try_expression";
    if shortcut {
        return true;
    }
    let deferred = node.kind() == "closure_expression";
    if deferred {
        return false;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor).any(propagates)
}
fn divides(node: Node<'_>) -> bool {
    let binary = node.kind() == "binary_expression";
    let operator = node
        .child_by_field_name("operator")
        .map(|operator| operator.kind());
    let divisor = node
        .child_by_field_name("right")
        .map(|divisor| divisor.kind());
    let literal = divisor.is_some_and(|kind| kind.ends_with("_literal"));
    let unchecked = binary && operator == Some("/") && !literal;
    if unchecked {
        return true;
    }
    let deferred = node.kind() == "closure_expression";
    if deferred {
        return false;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor).any(divides)
}
