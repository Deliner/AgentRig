use crate::structural_lint::languages::neutral_metrics::{Code, sole, text};
use tree_sitter::Node;

pub(super) fn method<'t, 's>(call: Node<'t>, source: Code<'s>) -> Option<(&'s str, Node<'t>)> {
    let invoked = call.kind() == "call_expression";
    let function = invoked
        .then(|| call.child_by_field_name("function"))
        .flatten()?;
    let field = function.child_by_field_name("field")?;
    Some((text(field, source), function.child_by_field_name("value")?))
}
pub(super) fn sole_argument(call: Node<'_>) -> Option<Node<'_>> {
    sole(call.child_by_field_name("arguments")?)
}
pub(super) fn calls(node: Node<'_>) -> bool {
    let mut cursor = node.walk();
    let found = node.kind() == "call_expression";
    found || node.named_children(&mut cursor).any(calls)
}
