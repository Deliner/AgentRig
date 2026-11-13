use super::method;
use crate::structural_lint::languages::neutral_metrics::{Code, text};
use crate::structural_lint::languages::rust::exemptions::last_segment;
use std::iter::successors;
use tree_sitter::Node;

pub(super) fn hasher(identifier: Node<'_>, source: Code<'_>) -> Option<&'static str> {
    let segment = text(identifier, source) == "DefaultHasher"
        && identifier
            .parent()
            .is_some_and(|parent| parent.kind() == "scoped_identifier");
    let imported = segment
        && successors(identifier.parent(), Node::parent)
            .any(|node| node.kind() == "use_declaration");
    let expression = segment && !imported;
    expression.then_some("DefaultHasher")
}
pub(super) fn named_type(call: Node<'_>, source: Code<'_>) -> Option<&'static str> {
    let callee = call.child_by_field_name("function")?;
    let path = callee
        .child_by_field_name("function")
        .filter(|_| callee.kind() == "generic_function")
        .unwrap_or(callee);
    let named = matches!(path.kind(), "identifier" | "scoped_identifier");
    let name = named.then(|| last_segment(path, source))?;
    let empty = call.child_by_field_name("arguments")?.named_child_count() == 0;
    match name {
        "type_name" => empty.then_some("type_name"),
        "type_name_of_val" => Some("type_name_of_val"),
        _ => None,
    }
}
pub(super) fn sort(call: Node<'_>, source: Code<'_>) -> Option<&'static str> {
    let (name, _) = method(call, source)?;
    match name {
        "sort_unstable_by" => Some("sort_unstable_by"),
        "sort_unstable_by_key" => Some("sort_unstable_by_key"),
        _ => None,
    }
}
