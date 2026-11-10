use super::context::in_test_context;
use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use tree_sitter::Node;

// DECISION: D100
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let pair = segment(node, "path") && segment(node, "name");
    let single = !pair;
    if single {
        return;
    }
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    let reached = node
        .parent()
        .and_then(|parent| parent.child_by_field_name("name"))
        .map_or("super::super", |name| text(name, source));
    output.push(Measurement {
        kind: Kind::Reference(Reference::SuperSuperReach),
        line: node.start_position().row + 1,
        symbol: reached.into(),
        actual: 1,
    });
}
fn segment(node: Node<'_>, field: &str) -> bool {
    node.child_by_field_name(field)
        .is_some_and(|child| child.kind() == "super")
}
