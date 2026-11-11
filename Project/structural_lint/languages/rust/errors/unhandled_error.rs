use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::context::{in_test_context, last_segment};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use tree_sitter::Node;

const EXPECTED_ERRORS: &[&str] = &[
    "try_lock",
    "try_read",
    "try_write",
    "try_recv",
    "try_send",
    "try_borrow",
    "try_borrow_mut",
    "recv",
    "recv_timeout",
    "binary_search",
    "binary_search_by",
    "binary_search_by_key",
    "compare_exchange",
    "compare_exchange_weak",
    "fetch_update",
    "try_unwrap",
    "downcast",
];

pub(super) fn ok_without_else(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let open = node.child_by_field_name("alternative").is_none();
    let Some(condition) = node.child_by_field_name("condition").filter(|_| open) else {
        return;
    };
    let unhandled = let_conditions(condition)
        .into_iter()
        .any(|binding| unhandled_ok(binding, source));
    let allowed = !unhandled || justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::UnhandledError),
        line: node.start_position().row + 1,
        symbol: "Ok".into(),
        actual: 1,
    });
}
pub(super) fn let_conditions(condition: Node<'_>) -> Vec<Node<'_>> {
    match condition.kind() {
        "let_condition" => vec![condition],
        "let_chain" => {
            let mut cursor = condition.walk();
            condition
                .named_children(&mut cursor)
                .flat_map(let_conditions)
                .collect()
        }
        _ => Vec::new(),
    }
}
pub(super) fn variant_pattern(binding: Node<'_>, variant: &str, source: Code<'_>) -> bool {
    binding.kind() == "tuple_struct_pattern"
        && binding
            .child_by_field_name("type")
            .is_some_and(|path| last_segment(path, source) == variant)
}
fn unhandled_ok(binding: Node<'_>, source: Code<'_>) -> bool {
    let ok = binding
        .child_by_field_name("pattern")
        .is_some_and(|pattern| variant_pattern(pattern, "Ok", source));
    ok && binding
        .child_by_field_name("value")
        .is_some_and(|value| !expected_error(value, source))
}
fn expected_error(value: Node<'_>, source: Code<'_>) -> bool {
    let mut current = value;
    while matches!(current.kind(), "await_expression" | "try_expression") {
        let Some(inner) = current.named_child(0) else {
            return false;
        };
        current = inner;
    }
    current.kind() == "call_expression"
        && current
            .child_by_field_name("function")
            .and_then(|function| callee(function, source))
            .is_some_and(|name| EXPECTED_ERRORS.contains(&name))
}
fn callee<'a>(function: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    match function.kind() {
        "generic_function" => callee(function.child_by_field_name("function")?, source),
        "field_expression" => function
            .child_by_field_name("field")
            .map(|field| text(field, source)),
        _ => Some(last_segment(function, source)),
    }
}
