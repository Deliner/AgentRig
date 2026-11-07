use tree_sitter::Node;

// DECISION: D118
pub(super) fn scope_boundary(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "closure_expression" | "async_block" | "function_item"
    )
}
pub(super) fn in_async_context(node: Node<'_>) -> bool {
    let mut current = node.parent();
    while let Some(ancestor) = current {
        let bounded = scope_boundary(ancestor);
        if bounded {
            return asynchronous(ancestor);
        }
        current = ancestor.parent();
    }
    false
}
fn asynchronous(scope: Node<'_>) -> bool {
    let mut cursor = scope.walk();
    let modified = scope
        .children(&mut cursor)
        .filter(|child| child.kind() == "function_modifiers")
        .any(|modifiers| {
            let mut inner = modifiers.walk();
            modifiers
                .children(&mut inner)
                .any(|token| token.kind() == "async")
        });
    scope.kind() == "async_block" || modified
}
