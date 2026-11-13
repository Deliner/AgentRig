mod false_pass;
mod shared_resource;
mod test_placement;
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement};
use tree_sitter::Node;

pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    false_pass::inspect(node, source, output);
    match node.kind() {
        "mod_item" => test_placement::inspect(node, source, output),
        "call_expression" => shared_resource::inspect(node, source, output),
        _ => {}
    }
}
