mod hidden_exit;
mod length_subtraction;
mod range_chain;
mod stale_check;
mod unlabeled_exit;
mod variant_equality;
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement};
use tree_sitter::Node;

pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    match node.kind() {
        "break_expression" | "continue_expression" => unlabeled_exit::inspect(node, source, output),
        "binary_expression" => {
            length_subtraction::inspect(node, source, output);
            variant_equality::comparison(node, source, output);
        }
        "macro_invocation" => variant_equality::matched(node, source, output),
        "macro_rule" => hidden_exit::inspect(node, source, output),
        "call_expression" => stale_check::inspect(node, source, output),
        "if_expression" => range_chain::chain(node, source, output),
        "match_expression" => range_chain::guards(node, source, output),
        _ => {}
    }
}
