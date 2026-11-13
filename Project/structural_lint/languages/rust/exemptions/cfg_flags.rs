use super::attribute_path;
use crate::structural_lint::languages::neutral_metrics::{Code, text};
use tree_sitter::Node;

pub(super) fn cfg_marker(node: Node<'_>, source: Code<'_>, flag: &str) -> bool {
    cfg_arguments(node, source).is_some_and(|arguments| cfg_requires(arguments, source, flag))
}
pub fn cfg_excludes(node: Node<'_>, source: Code<'_>, flag: &str) -> bool {
    cfg_arguments(node, source).is_some_and(|arguments| {
        predicate_groups(arguments)
            .iter()
            .any(|group| forbids(group, source, flag))
    })
}
fn cfg_arguments<'a>(node: Node<'a>, source: Code<'_>) -> Option<Node<'a>> {
    let path = attribute_path(node)?;
    let cfg = path.kind() == "identifier" && text(path, source) == "cfg";
    let other = !cfg;
    if other {
        return None;
    }
    path.parent()?.child_by_field_name("arguments")
}
pub(super) fn cfg_requires(arguments: Node<'_>, source: Code<'_>, flag: &str) -> bool {
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
fn group_requires(group: &[Node<'_>], source: Code<'_>, flag: &str) -> bool {
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
fn forbids(group: &[Node<'_>], source: Code<'_>, flag: &str) -> bool {
    let Some((name, groups)) = combinator(group, source) else {
        return false;
    };
    let populated = !groups.is_empty();
    match name {
        "not" => groups.iter().any(|inner| implied(inner, source, flag)),
        "all" => groups.iter().any(|inner| forbids(inner, source, flag)),
        "any" => populated && groups.iter().all(|inner| forbids(inner, source, flag)),
        _ => false,
    }
}
fn implied(group: &[Node<'_>], source: Code<'_>, flag: &str) -> bool {
    let Some((name, groups)) = combinator(group, source) else {
        let bare = group.len() == 1;
        return bare && group.iter().all(|head| text(*head, source) == flag);
    };
    let populated = !groups.is_empty();
    match name {
        "not" => groups.iter().any(|inner| forbids(inner, source, flag)),
        "all" => populated && groups.iter().all(|inner| implied(inner, source, flag)),
        "any" => groups.iter().any(|inner| implied(inner, source, flag)),
        _ => false,
    }
}
fn combinator<'a, 's>(
    group: &[Node<'a>],
    source: Code<'s>,
) -> Option<(&'s str, Vec<Vec<Node<'a>>>)> {
    let (head, rest) = group.split_first()?;
    let inner = rest.iter().find(|node| node.kind() == "token_tree")?;
    Some((text(*head, source), predicate_groups(*inner)))
}
