use crate::structural_lint::languages::neutral_metrics::{Code, descendants, text};
use tree_sitter::Node;

pub fn attributed(
    item: Node<'_>,
    source: Code<'_>,
    marked: impl Fn(Node<'_>, Code<'_>) -> bool,
) -> bool {
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
pub fn listed(attribute_item: Node<'_>, source: Code<'_>, head: &str, wanted: &[&str]) -> bool {
    descendants(attribute_item).into_iter().any(|node| {
        let named = node.kind() == "identifier" && text(node, source) == head;
        named
            && node
                .next_named_sibling()
                .filter(|list| list.kind() == "token_tree")
                .is_some_and(|list| {
                    descendants(list).into_iter().any(|token| {
                        token.kind() == "identifier" && wanted.contains(&text(token, source))
                    })
                })
    })
}
pub fn derives(item: Node<'_>, source: Code<'_>, wanted: &[&str]) -> bool {
    attributed(item, source, |attribute, source| {
        listed(attribute, source, "derive", wanted)
    })
}
pub fn attribute_path(node: Node<'_>) -> Option<Node<'_>> {
    node.named_children(&mut node.walk())
        .find(|child| child.kind() == "attribute")?
        .named_child(0)
}
