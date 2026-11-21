use super::call_syntax::calls;
use crate::structural_lint::languages::neutral_metrics::Measurement;
use crate::structural_lint::rule_identities::{Kind, Reference};
use tree_sitter::Node;

// DECISION: D205
pub(super) fn inspect(node: Node<'_>, output: &mut Vec<Measurement>) {
    let flagged = node.kind() == "index_expression" && listed(node).is_some_and(calls);
    if flagged {
        output.push(Measurement {
            kind: Kind::Reference(Reference::IndexedArrayLiteral),
            line: node.start_position().row + 1,
            symbol: "[..][index]".into(),
            actual: 1,
        });
    }
}
fn listed(indexed: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = indexed.walk();
    let mut parts = indexed
        .named_children(&mut cursor)
        .filter(|part| !part.is_extra());
    let array = parts.next()?;
    let index = parts.next()?;
    let array_literal = array.kind() == "array_expression";
    let elements = array_literal && array.child_by_field_name("length").is_none();
    let whole = index.kind() == "range_expression" && index.named_child_count() == 0;
    let selected = index.kind() != "integer_literal" && !whole;
    (elements && selected).then_some(array)
}
