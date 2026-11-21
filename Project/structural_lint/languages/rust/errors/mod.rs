mod call_syntax;
mod collapsed_chain;
mod discarded_result;
mod error_verdict;
mod process_success;
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement};
use tree_sitter::Node;

pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    match node.kind() {
        "call_expression" => collapsed_chain::inspect(node, source, output),
        "match_arm" => error_verdict::match_arm(node, source, output),
        "if_expression" => error_verdict::if_let(node, source, output),
        "expression_statement" | "let_declaration" => {
            discarded_result::inspect(node, source, output)
        }
        // DECISION: D126
        "function_item" => process_success::inspect(node, source, output),
        _ => {}
    }
}
