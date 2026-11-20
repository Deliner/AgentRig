use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, text};
use crate::structural_lint::languages::rust::exemptions::{ENTRY_FILES, attributed, test_marker};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::structure::Structure;
use tree_sitter::Node;

// DECISION: D224
pub(super) fn inspect(attribute: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let first = test_marker(attribute, source) && !attributed(attribute, source, test_marker);
    let Some(item) = first.then(|| decorated(attribute)).flatten() else {
        return;
    };
    let nested = std::iter::successors(item.parent(), Node::parent)
        .any(|outer| attributed(outer, source, test_marker));
    if nested {
        return;
    }
    let declaration = item.kind() == "mod_item" && item.child_by_field_name("body").is_none();
    let entry = source
        .path()
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| ENTRY_FILES.contains(&name));
    let declared = declaration && entry;
    if declared {
        return;
    }
    let symbol = item
        .child_by_field_name("name")
        .map_or(item.kind(), |name| text(name, source));
    output.push(Measurement {
        kind: Kind::Structure(Structure::TestInCodeFile),
        line: item.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn decorated(attribute: Node<'_>) -> Option<Node<'_>> {
    std::iter::successors(attribute.next_named_sibling(), Node::next_named_sibling).find(|node| {
        !matches!(
            node.kind(),
            "attribute_item" | "line_comment" | "block_comment"
        )
    })
}
