use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::context::in_test_context;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::ownership::Ownership;
use tree_sitter::Node;

// DECISION: D102
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let other = callee(function, source) != "remove_dir_all";
    if other {
        return;
    }
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Ownership(Ownership::RecursiveDeletion),
        line: node.start_position().row + 1,
        symbol: "remove_dir_all".into(),
        actual: 1,
    });
}
fn callee<'a>(function: Node<'_>, source: Code<'a>) -> &'a str {
    let named = function
        .child_by_field_name("field")
        .or_else(|| function.child_by_field_name("name"))
        .unwrap_or(function);
    text(named, source)
}
