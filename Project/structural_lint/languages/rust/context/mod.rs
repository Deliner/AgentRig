pub(super) mod async_code;
use crate::structural_lint::languages::measure::text;
use tree_sitter::Node;

pub(super) fn in_test_context(node: Node<'_>, source: &str) -> bool {
    let mut current = node;
    while let Some(parent) = current.parent() {
        let item = matches!(
            parent.kind(),
            "mod_item" | "function_item" | "function_signature_item"
        );
        let test = item && attributed(parent, source, test_marker);
        if test {
            return true;
        }
        current = parent;
    }
    false
}
// DECISION: D115
pub(super) fn in_debug_only_context(node: Node<'_>, source: &str) -> bool {
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
fn debug_branch(block: Node<'_>, source: &str) -> bool {
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
pub(super) fn attributed(item: Node<'_>, source: &str, marked: fn(Node<'_>, &str) -> bool) -> bool {
    let mut sibling = item.prev_named_sibling();
    while let Some(node) = sibling {
        let marker = node.kind() == "attribute_item" && marked(node, source);
        if marker {
            return true;
        }
        let decoration = matches!(
            node.kind(),
            "attribute_item" | "line_comment" | "block_comment"
        );
        let stop = !decoration;
        if stop {
            return false;
        }
        sibling = node.prev_named_sibling();
    }
    false
}
fn test_marker(node: Node<'_>, source: &str) -> bool {
    let named_test = attribute_path(node).is_some_and(|path| last_segment(path, source) == "test");
    named_test || cfg_marker(node, source, "test")
}
fn debug_marker(node: Node<'_>, source: &str) -> bool {
    cfg_marker(node, source, "debug_assertions")
}
fn attribute_path(node: Node<'_>) -> Option<Node<'_>> {
    node.named_children(&mut node.walk())
        .find(|child| child.kind() == "attribute")?
        .named_child(0)
}
fn cfg_marker(node: Node<'_>, source: &str, flag: &str) -> bool {
    let Some(path) = attribute_path(node) else {
        return false;
    };
    let cfg = path.kind() == "identifier" && text(path, source) == "cfg";
    cfg && path
        .parent()
        .and_then(|attribute| attribute.child_by_field_name("arguments"))
        .is_some_and(|arguments| cfg_requires(arguments, source, flag))
}
pub(super) fn last_segment<'a>(path: Node<'_>, source: &'a str) -> &'a str {
    if let Some(name) = path.child_by_field_name("name") {
        return text(name, source);
    }
    text(path, source)
}
fn cfg_requires(arguments: Node<'_>, source: &str, flag: &str) -> bool {
    predicate_groups(arguments)
        .iter()
        .any(|group| group_requires(group, source, flag))
}
fn predicate_groups<'a>(tree: Node<'a>) -> Vec<Vec<Node<'a>>> {
    let mut groups = Vec::new();
    let mut current = Vec::new();
    let mut cursor = tree.walk();
    for child in tree.children(&mut cursor) {
        match child.kind() {
            "(" | ")" => {}
            "," => groups.push(std::mem::take(&mut current)),
            _ => current.push(child),
        }
    }
    let trailing = !current.is_empty();
    if trailing {
        groups.push(current);
    }
    groups
}
fn group_requires(group: &[Node<'_>], source: &str, flag: &str) -> bool {
    let assignment = group.iter().any(|node| node.kind() == "=");
    if assignment {
        return false;
    }
    let Some((head, rest)) = group.split_first() else {
        return false;
    };
    let name = text(*head, source);
    let Some(inner) = rest.iter().find(|node| node.kind() == "token_tree") else {
        return name == flag;
    };
    match name {
        "all" => predicate_groups(*inner)
            .iter()
            .any(|group| group_requires(group, source, flag)),
        "any" => {
            let groups = predicate_groups(*inner);
            let populated = !groups.is_empty();
            populated
                && groups
                    .iter()
                    .all(|group| group_requires(group, source, flag))
        }
        _ => false,
    }
}
