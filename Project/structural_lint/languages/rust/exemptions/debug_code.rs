use super::attributed;
use super::cfg_flags::{cfg_marker, cfg_requires};
use crate::structural_lint::languages::neutral_metrics::{Code, text};
use tree_sitter::Node;

// DECISION: D115
pub fn in_debug_only_context(node: Node<'_>, source: Code<'_>) -> bool {
    let mut current = node;
    loop {
        let gated = attributed(current, source, debug_marker) || debug_branch(current, source);
        if gated {
            return true;
        }
        let Some(parent) = current.parent() else {
            return false;
        };
        current = parent;
    }
}
fn debug_branch(block: Node<'_>, source: Code<'_>) -> bool {
    let Some(branch) = block.parent() else {
        return false;
    };
    let Some(condition) = branch.child_by_field_name("condition") else {
        return false;
    };
    let consequence = branch.kind() == "if_expression"
        && branch.child_by_field_name("consequence") == Some(block);
    let cfg = consequence
        && condition.kind() == "macro_invocation"
        && condition
            .child_by_field_name("macro")
            .is_some_and(|path| text(path, source) == "cfg");
    cfg && condition
        .named_children(&mut condition.walk())
        .find(|child| child.kind() == "token_tree")
        .is_some_and(|tree| cfg_requires(tree, source, "debug_assertions"))
}
fn debug_marker(node: Node<'_>, source: Code<'_>) -> bool {
    cfg_marker(node, source, "debug_assertions")
}
