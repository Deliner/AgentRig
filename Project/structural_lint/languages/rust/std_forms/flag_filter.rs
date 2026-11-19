use super::{calls, method, sole_argument};
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, justified};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use tree_sitter::Node;

// DECISION: D208
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let computed =
        method(node, source).is_some_and(|(name, receiver)| name == "filter" && calls(receiver));
    let flagged = computed && sole_argument(node).is_some_and(ignores_the_value);
    let allowed = !flagged || justified(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::OuterFlagFilter),
        line: node.start_position().row + 1,
        symbol: "filter".into(),
        actual: 1,
    });
}
fn ignores_the_value(argument: Node<'_>) -> bool {
    let closure = argument.kind() == "closure_expression";
    let wildcard = argument
        .child_by_field_name("parameters")
        .is_some_and(|list| {
            list.child_count() == 3 && list.child(1).is_some_and(|only| only.kind() == "_")
        });
    let free = argument
        .child_by_field_name("body")
        .is_some_and(|body| !calls(body));
    closure && wildcard && free
}
