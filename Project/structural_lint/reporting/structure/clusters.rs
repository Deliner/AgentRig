use super::corpus::IDENTIFIERS;
use crate::structural_lint::languages::{Code, in_test_context, text};
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::Node;

const ITEMS: &[&str] = &[
    "function_item",
    "struct_item",
    "enum_item",
    "union_item",
    "trait_item",
    "impl_item",
    "type_item",
    "const_item",
    "static_item",
    "macro_definition",
];

pub fn count(root: Node<'_>, code: Code<'_>) -> usize {
    let mut cursor = root.walk();
    let items: Vec<(String, BTreeSet<String>)> = root
        .named_children(&mut cursor)
        .filter(|node| ITEMS.contains(&node.kind()))
        .filter_map(|node| item(node, code))
        .collect();
    let mut named: BTreeMap<&str, usize> = BTreeMap::new();
    for (index, (name, _)) in items.iter().enumerate() {
        named.entry(name).or_insert(index);
    }
    let mut owners: Vec<usize> = (0..items.len()).collect();
    for (index, (name, references)) in items.iter().enumerate() {
        let related = std::iter::once(name).chain(references);
        let targets = related.filter_map(|word| named.get(word.as_str()).copied());
        attach(&mut owners, index, targets);
    }
    let mut sizes: BTreeMap<usize, usize> = BTreeMap::new();
    for index in 0..items.len() {
        *sizes.entry(owner(&owners, index)).or_default() += 1;
    }
    sizes.values().filter(|&&size| size >= 2).count()
}

fn item(node: Node<'_>, code: Code<'_>) -> Option<(String, BTreeSet<String>)> {
    let implementation = node.kind() == "impl_item";
    let named = if implementation {
        node.child_by_field_name("type").and_then(type_name)
    } else {
        node.child_by_field_name("name")
    }?;
    let test = in_test_context(named, code);
    if test {
        return None;
    }
    let mut references = BTreeSet::new();
    let mut pending = vec![node];
    while let Some(current) = pending.pop() {
        let identifier = IDENTIFIERS.contains(&current.kind());
        if identifier {
            references.insert(text(current, code).to_owned());
        }
        let mut cursor = current.walk();
        pending.extend(current.named_children(&mut cursor));
    }
    Some((text(named, code).to_owned(), references))
}

fn type_name(node: Node<'_>) -> Option<Node<'_>> {
    let mut pending = vec![node];
    while let Some(current) = pending.pop() {
        let type_identifier = current.kind() == "type_identifier";
        if type_identifier {
            return Some(current);
        }
        let mut cursor = current.walk();
        let children: Vec<Node<'_>> = current.named_children(&mut cursor).collect();
        pending.extend(children.into_iter().rev());
    }
    None
}

fn attach(owners: &mut [usize], item: usize, targets: impl Iterator<Item = usize>) {
    for target in targets {
        let (left, right) = (owner(owners, item), owner(owners, target));
        if let Some(slot) = owners.get_mut(left.max(right)) {
            *slot = left.min(right);
        }
    }
}

fn owner(owners: &[usize], index: usize) -> usize {
    let mut current = index;
    while let Some(&next) = owners.get(current) {
        let root = next == current;
        if root {
            break;
        }
        current = next;
    }
    current
}
