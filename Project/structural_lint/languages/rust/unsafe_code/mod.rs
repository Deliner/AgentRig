mod send_bound;
mod unchecked_operation;
mod unsafe_site;
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement};
use tree_sitter::Node;

pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    unsafe_site::inspect(node, source, output);
    match node.kind() {
        "impl_item" => send_bound::inspect(node, source, output),
        "call_expression" => unchecked_operation::inspect(node, source, output),
        _ => {}
    }
}
