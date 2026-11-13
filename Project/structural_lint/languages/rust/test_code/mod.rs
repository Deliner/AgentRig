mod false_pass;
mod test_placement;
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement};
use tree_sitter::Node;

pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    false_pass::inspect(node, source, output);
    let module = node.kind() == "mod_item";
    if module {
        test_placement::inspect(node, source, output);
    }
}
