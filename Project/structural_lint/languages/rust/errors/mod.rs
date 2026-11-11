mod error_text;
mod silent_error;
mod unhandled_error;
use crate::structural_lint::languages::measure::{Code, Measurement};
use tree_sitter::Node;

pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    match node.kind() {
        "call_expression" => {
            silent_error::collapsed_chain(node, source, output);
            error_text::text_branch(node, source, output);
        }
        "match_arm" => silent_error::verdict_arm(node, source, output),
        "if_expression" => {
            silent_error::verdict_if(node, source, output);
            unhandled_error::ok_without_else(node, source, output);
        }
        // DECISION: D126
        "function_item" => silent_error::exit_verdict(node, source, output),
        _ => {}
    }
}
