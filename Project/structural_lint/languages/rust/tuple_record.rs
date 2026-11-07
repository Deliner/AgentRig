use super::context::in_test_context;
use crate::structural_lint::languages::measure::{Measurement, justified, text};
use crate::structural_lint::rules::kind::Kind;
use tree_sitter::Node;

// DECISION: D088
// DECISION: D101
pub(super) fn record_signature(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let named = matches!(node.kind(), "function_item" | "function_signature_item");
    let tuple = annotations(node).any(|annotation| {
        let top = named && annotation.kind() == "tuple_type";
        top || nested_record(annotation, source)
    });
    let plain = !tuple;
    if plain {
        return;
    }
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::TupleAsRecord,
        line: node.start_position().row + 1,
        symbol: "signature".into(),
        actual: 1,
    });
}
fn annotations<'a>(node: Node<'a>) -> impl Iterator<Item = Node<'a>> {
    let parameters = node
        .child_by_field_name("parameters")
        .into_iter()
        .flat_map(|parameters| {
            let mut cursor = parameters.walk();
            parameters
                .named_children(&mut cursor)
                .filter_map(|parameter| parameter.child_by_field_name("type"))
                .collect::<Vec<_>>()
        });
    parameters.chain(node.child_by_field_name("return_type"))
}
fn nested_record(node: Node<'_>, source: &str) -> bool {
    let container = matches!(node.kind(), "generic_type" | "array_type");
    if container {
        return contained_record(node, source);
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| nested_record(child, source))
}
fn contained_record(node: Node<'_>, source: &str) -> bool {
    let record = node.kind() == "tuple_type" && unnamed_record(node, source);
    if record {
        return true;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| contained_record(child, source))
}
fn unnamed_record(tuple: Node<'_>, source: &str) -> bool {
    let mut cursor = tuple.walk();
    tuple.named_child_count() >= 3
        && tuple
            .named_children(&mut cursor)
            .any(|element| scalar(element, source))
}
fn scalar(node: Node<'_>, source: &str) -> bool {
    match node.kind() {
        "primitive_type" => true,
        "type_identifier" => text(node, source) == "String",
        "reference_type" => node
            .child_by_field_name("type")
            .is_some_and(|inner| inner.kind() == "primitive_type"),
        _ => false,
    }
}
