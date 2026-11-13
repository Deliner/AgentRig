use super::cfg_flags::cfg_marker;
use super::{attribute_path, attributed, last_segment};
use crate::structural_lint::languages::neutral_metrics::Code;
use tree_sitter::Node;

// DECISION: D148
pub fn in_test_context(node: Node<'_>, source: Code<'_>) -> bool {
    let mut current = node;
    while let Some(parent) = current.parent() {
        let item = matches!(
            parent.kind(),
            "mod_item" | "function_item" | "function_signature_item" | "macro_definition"
        );
        let test = item && attributed(parent, source, test_marker);
        if test {
            return true;
        }
        current = parent;
    }
    false
}
pub fn test_marker(node: Node<'_>, source: Code<'_>) -> bool {
    let named_test = attribute_path(node).is_some_and(|path| last_segment(path, source) == "test");
    named_test || cfg_marker(node, source, "test")
}
