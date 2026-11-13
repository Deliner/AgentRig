pub(super) mod async_code;
mod cfg_flags;
mod debug_code;
mod test_code;

use crate::structural_lint::languages::neutral_metrics::{Code, descendants, justified_line, text};
pub(super) use cfg_flags::cfg_excludes;
pub(super) use debug_code::in_debug_only_context;
pub(crate) use test_code::in_test_context;
pub(super) use test_code::test_marker;
use tree_sitter::Node;

pub(super) fn attributed(
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
pub(super) fn justified_item(item: Node<'_>, source: Code<'_>) -> bool {
    let mut anchor = item;
    while let Some(previous) = anchor.prev_sibling() {
        let other = previous.kind() != "attribute_item";
        if other {
            break;
        }
        anchor = previous;
    }
    justified_line(anchor.start_position().row, source.as_str())
}
pub(super) fn trait_impl_member(item: Node<'_>) -> bool {
    item.parent()
        .and_then(|list| list.parent())
        .is_some_and(|owner| {
            owner.kind() == "impl_item" && owner.child_by_field_name("trait").is_some()
        })
}
pub(super) fn listed(
    attribute_item: Node<'_>,
    source: Code<'_>,
    head: &str,
    wanted: &[&str],
) -> bool {
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
pub(super) fn derives(item: Node<'_>, source: Code<'_>, wanted: &[&str]) -> bool {
    attributed(item, source, |attribute, source| {
        listed(attribute, source, "derive", wanted)
    })
}
pub(super) fn attribute_path(node: Node<'_>) -> Option<Node<'_>> {
    node.named_children(&mut node.walk())
        .find(|child| child.kind() == "attribute")?
        .named_child(0)
}
pub(super) fn last_segment<'a>(path: Node<'_>, source: Code<'a>) -> &'a str {
    if let Some(name) = path.child_by_field_name("name") {
        return text(name, source);
    }
    text(path, source)
}
pub(super) fn zero_literal(node: Node<'_>, source: Code<'_>) -> bool {
    let value = text(node, source);
    let digits = match node.kind() {
        "integer_literal" => {
            let radix = ["0x", "0o", "0b"]
                .into_iter()
                .find_map(|prefix| value.strip_prefix(prefix));
            radix.unwrap_or(value).split(['u', 'i']).next()
        }
        "float_literal" => value.split(['e', 'E', 'f']).next(),
        _ => None,
    };
    digits.is_some_and(|digits| digits.chars().all(|digit| matches!(digit, '0' | '_' | '.')))
}
pub(super) fn type_name<'a>(node: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    match node.kind() {
        "generic_type" => type_name(node.child_by_field_name("type")?, source),
        _ => plain_name(node, source),
    }
}
pub(super) fn plain_name<'a>(node: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    match node.kind() {
        "type_identifier" => Some(text(node, source)),
        "scoped_type_identifier" => Some(text(node.child_by_field_name("name")?, source)),
        _ => None,
    }
}
