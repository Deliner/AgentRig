use super::text;
use tree_sitter::Node;

pub(super) fn in_test_context(node: Node<'_>, source: &str) -> bool {
    let mut current = node;
    while let Some(parent) = current.parent() {
        let item = matches!(
            parent.kind(),
            "mod_item" | "function_item" | "function_signature_item"
        );
        let test = item && test_attributed(parent, source);
        if test {
            return true;
        }
        current = parent;
    }
    false
}
fn test_attributed(item: Node<'_>, source: &str) -> bool {
    let mut sibling = item.prev_named_sibling();
    while let Some(node) = sibling {
        let marker = node.kind() == "attribute_item" && test_marker(node, source);
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
    let Some(attribute) = node
        .named_children(&mut node.walk())
        .find(|child| child.kind() == "attribute")
    else {
        return false;
    };
    let Some(path) = attribute.named_child(0) else {
        return false;
    };
    let named_test = last_segment(path, source) == "test";
    if named_test {
        return true;
    }
    let cfg = path.kind() == "identifier" && text(path, source) == "cfg";
    if cfg {
        return attribute
            .child_by_field_name("arguments")
            .is_some_and(|arguments| cfg_requires_test(arguments, source));
    }
    false
}
fn last_segment<'a>(path: Node<'_>, source: &'a str) -> &'a str {
    if let Some(name) = path.child_by_field_name("name") {
        return text(name, source);
    }
    text(path, source)
}
fn cfg_requires_test(arguments: Node<'_>, source: &str) -> bool {
    predicate_groups(arguments)
        .iter()
        .any(|group| group_requires_test(group, source))
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
fn group_requires_test(group: &[Node<'_>], source: &str) -> bool {
    let assignment = group.iter().any(|node| node.kind() == "=");
    if assignment {
        return false;
    }
    let Some((head, rest)) = group.split_first() else {
        return false;
    };
    let name = text(*head, source);
    let Some(inner) = rest.iter().find(|node| node.kind() == "token_tree") else {
        return name == "test";
    };
    match name {
        "all" => predicate_groups(*inner)
            .iter()
            .any(|group| group_requires_test(group, source)),
        "any" => {
            let groups = predicate_groups(*inner);
            let populated = !groups.is_empty();
            populated
                && groups
                    .iter()
                    .all(|group| group_requires_test(group, source))
        }
        _ => false,
    }
}
