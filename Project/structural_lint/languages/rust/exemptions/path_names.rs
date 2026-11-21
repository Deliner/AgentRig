use crate::structural_lint::languages::neutral_metrics::{Code, text};
use tree_sitter::Node;

pub fn last_segment<'a>(path: Node<'_>, source: Code<'a>) -> &'a str {
    if let Some(name) = path.child_by_field_name("name") {
        return text(name, source);
    }
    text(path, source)
}
pub fn type_name<'a>(node: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    match node.kind() {
        "generic_type" => type_name(node.child_by_field_name("type")?, source),
        _ => plain_name(node, source),
    }
}
pub fn plain_name<'a>(node: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    match node.kind() {
        "type_identifier" => Some(text(node, source)),
        "scoped_type_identifier" => Some(text(node.child_by_field_name("name")?, source)),
        _ => None,
    }
}
