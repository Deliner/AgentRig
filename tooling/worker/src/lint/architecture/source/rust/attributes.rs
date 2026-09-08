use super::{Source, Target, compact, scope};
use tree_sitter::Node;

pub(super) fn inspect(source: &mut Source<'_>, node: Node<'_>) {
    let Some(attribute) = node
        .named_children(&mut node.walk())
        .find(|child| child.kind() == "attribute")
    else {
        return;
    };
    let name = attribute
        .named_child(0)
        .map(|name| source.text(name))
        .unwrap_or("");
    match name {
        "derive" => derives(source, attribute),
        "serde" => serialization(source, attribute),
        "cfg" => {
            let unsupported = !attribute
                .child_by_field_name("arguments")
                .is_some_and(|args| compact(source.text(args)) == "(test)");
            if unsupported {
                source.unsupported(node, "Rust conditional expansion is not analyzed");
            }
        }
        "default" => {}
        "path" => validate_module_path(source, node, attribute),
        _ => super::attribute(source, node),
    }
}

fn validate_module_path(source: &mut Source<'_>, node: Node<'_>, attribute: Node<'_>) {
    let mut next = node.next_named_sibling();
    while next.is_some_and(|item| item.is_extra() || item.kind() == "attribute_item") {
        next = next.and_then(|item| item.next_named_sibling());
    }
    let supported = node
        .parent()
        .is_some_and(|parent| parent.kind() == "source_file")
        && next.is_some_and(|item| {
            item.kind() == "mod_item" && item.child_by_field_name("body").is_none()
        })
        && attribute
            .child_by_field_name("value")
            .and_then(|value| super::string_literal(source, value))
            .is_some();
    let unsupported = !supported;
    if unsupported {
        source.unsupported(
            node,
            "Rust path attributes require a literal on a file-level external module",
        );
    }
}

pub(super) fn module_path(source: &mut Source<'_>, node: Node<'_>) -> Option<String> {
    let mut previous = node.prev_named_sibling();
    let mut result = None;
    while let Some(item) = previous {
        let end_of_attributes = !item.is_extra() && item.kind() != "attribute_item";
        if end_of_attributes {
            break;
        }
        if let Some(attribute) = item.named_child(0) {
            let path = attribute
                .named_child(0)
                .is_some_and(|name| source.text(name) == "path");
            if path {
                let duplicate = result.is_some();
                if duplicate {
                    source.unsupported(item, "duplicate Rust module path attributes are ambiguous");
                }
                result = attribute
                    .child_by_field_name("value")
                    .and_then(|value| super::string_literal(source, value));
            }
        }
        previous = item.prev_named_sibling();
    }
    result
}

fn groups(attribute: Node<'_>) -> Vec<Vec<Node<'_>>> {
    let Some(arguments) = attribute.child_by_field_name("arguments") else {
        return Vec::new();
    };
    let mut groups = vec![Vec::new()];
    for child in arguments
        .children(&mut arguments.walk())
        .filter(|child| !child.is_extra())
    {
        match child.kind() {
            "(" | ")" => {}
            "," => groups.push(Vec::new()),
            _ => groups.last_mut().unwrap().push(child),
        }
    }
    groups
        .into_iter()
        .filter(|group| !group.is_empty())
        .collect()
}

fn derives(source: &mut Source<'_>, attribute: Node<'_>) {
    for group in groups(attribute) {
        let path = group
            .iter()
            .map(|node| compact(source.text(*node)))
            .collect();
        super::record(
            source,
            attribute,
            Target::RustDerive {
                path,
                scope: scope(source, attribute),
            },
        );
    }
}

fn serialization(source: &mut Source<'_>, attribute: Node<'_>) {
    for group in groups(attribute) {
        let key = source.text(group[0]);
        let flag = group.len() == 1
            && matches!(
                key,
                "default" | "deny_unknown_fields" | "flatten" | "skip_deserializing" | "untagged"
            );
        if flag {
            continue;
        }
        let value = serde_value(source, &group);
        match value {
            Some((path, true)) => super::record(
                source,
                attribute,
                Target::RustPath {
                    path,
                    scope: scope(source, attribute),
                },
            ),
            Some((_, false)) => {}
            None => source.unsupported(
                attribute,
                "Serde attribute requires supported literal metadata or callback path",
            ),
        }
    }
}

fn serde_value(source: &Source<'_>, group: &[Node<'_>]) -> Option<(String, bool)> {
    let [key, equals, value] = group else {
        return None;
    };
    let invalid = equals.kind() != "=";
    if invalid {
        return None;
    }
    let key = source.text(*key);
    let callback = matches!(
        key,
        "default" | "skip_serializing_if" | "deserialize_with" | "serialize_with"
    );
    let label = matches!(key, "rename" | "rename_all" | "alias");
    let supported = callback || label;
    if supported {
        return super::string_literal(source, *value).map(|value| (value, callback));
    }
    None
}
