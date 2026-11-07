use super::context::async_code::scope_boundary;
use super::context::{in_test_context, last_segment};
use crate::structural_lint::languages::measure::{Measurement, justified, text};
use crate::structural_lint::rules::kind::Kind;
use tree_sitter::Node;

// DECISION: D117
pub(super) fn inspect(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(name) = guard(node, source) else {
        return;
    };
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    let held = held_across_await(node, name, source);
    if held {
        output.push(Measurement {
            kind: Kind::GuardAcrossAwait,
            line: node.start_position().row + 1,
            symbol: name.into(),
            actual: 1,
        });
    }
}
fn guard<'a>(binding: Node<'_>, source: &'a str) -> Option<&'a str> {
    let pattern = binding.child_by_field_name("pattern")?;
    let value = binding.child_by_field_name("value")?;
    let named = pattern.kind() == "identifier";
    let locked = named && locking(value, source) && !awaits(value);
    locked.then(|| text(pattern, source))
}
fn locking(value: Node<'_>, source: &str) -> bool {
    let mut current = Some(value);
    while let Some(node) = current {
        let (receiver, name) = match node.kind() {
            "try_expression" => (node.named_child(0), "?"),
            _ => method(node, source).map_or((None, ""), |(inner, name)| (Some(inner), name)),
        };
        let bare = node
            .child_by_field_name("arguments")
            .is_some_and(|arguments| arguments.named_child_count() == 0);
        let lock = bare && matches!(name, "lock" | "read" | "write");
        if lock {
            return true;
        }
        let unwrapped = matches!(name, "?" | "unwrap" | "expect" | "unwrap_or_else");
        current = receiver.filter(|_| unwrapped);
    }
    false
}
fn method<'a, 'b>(call: Node<'a>, source: &'b str) -> Option<(Node<'a>, &'b str)> {
    let function = call.child_by_field_name("function")?;
    let field = function.child_by_field_name("field")?;
    let receiver = function.child_by_field_name("value")?;
    let selected = call.kind() == "call_expression" && function.kind() == "field_expression";
    selected.then(|| (receiver, text(field, source)))
}
fn held_across_await(binding: Node<'_>, name: &str, source: &str) -> bool {
    let scoped = binding
        .parent()
        .is_some_and(|block| block.kind() == "block");
    let mut next = binding.next_named_sibling().filter(|_| scoped);
    while let Some(statement) = next {
        let released = dropped(statement, name, source);
        if released {
            return false;
        }
        let suspended = awaits(statement);
        if suspended {
            return true;
        }
        next = statement.next_named_sibling();
    }
    false
}
fn dropped(statement: Node<'_>, name: &str, source: &str) -> bool {
    let call = statement.named_child(0);
    let function = call.and_then(|call| call.child_by_field_name("function"));
    let arguments = call.and_then(|call| call.child_by_field_name("arguments"));
    let path = function.filter(|path| matches!(path.kind(), "identifier" | "scoped_identifier"));
    let drop = path.is_some_and(|path| last_segment(path, source) == "drop");
    let guard = arguments.is_some_and(|arguments| {
        let single = arguments.named_child_count() == 1;
        single
            && arguments
                .named_child(0)
                .is_some_and(|value| text(value, source) == name)
    });
    statement.kind() == "expression_statement" && drop && guard
}
fn awaits(node: Node<'_>) -> bool {
    let nested = scope_boundary(node);
    if nested {
        return false;
    }
    let keyword = node.kind() == "await";
    let mut cursor = node.walk();
    keyword || node.children(&mut cursor).any(awaits)
}
