mod length_subtraction;
mod unlabeled_exit;
use crate::structural_lint::languages::measure::Measurement;
use tree_sitter::Node;

pub(super) fn inspect(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    match node.kind() {
        "break_expression" | "continue_expression" => unlabeled_exit::inspect(node, source, output),
        "binary_expression" => length_subtraction::inspect(node, source, output),
        _ => {}
    }
}
