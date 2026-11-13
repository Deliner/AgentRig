use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, descendants, text};
use crate::structural_lint::languages::rust::exemptions::{
    in_test_context, justified_item, type_name,
};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::signature::Signature;
use tree_sitter::Node;

const MIRRORS: &[&str] = &["PartialEq", "Hash", "PartialOrd", "Ord", "Clone"];

// DECISION: D156
pub(super) fn inspect(implementation: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let contract = implementation
        .child_by_field_name("trait")
        .and_then(|path| type_name(path, source))
        .filter(|name| MIRRORS.contains(name));
    let owner = implementation
        .child_by_field_name("type")
        .and_then(|path| type_name(path, source));
    let (Some(contract), Some(owner), Some(body)) =
        (contract, owner, implementation.child_by_field_name("body"))
    else {
        return;
    };
    let nodes = descendants(body);
    let accessed = nodes.iter().any(|node| self_field(*node));
    let complete = nodes.iter().any(|node| exhaustive(*node, owner, source));
    let covered = !accessed || complete;
    let allowed = covered
        || in_test_context(implementation, source)
        || justified_item(implementation, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Signature(Signature::PartialImpl),
        line: implementation.start_position().row + 1,
        symbol: format!("{contract} for {owner}"),
        actual: 1,
    });
}
fn self_field(node: Node<'_>) -> bool {
    node.kind() == "field_expression"
        && node
            .child_by_field_name("value")
            .is_some_and(|value| value.kind() == "self")
}
fn exhaustive(node: Node<'_>, owner: &str, source: Code<'_>) -> bool {
    let (path, list) = match node.kind() {
        "struct_pattern" | "tuple_struct_pattern" => (node.child_by_field_name("type"), Some(node)),
        "struct_expression" => (
            node.child_by_field_name("name"),
            node.child_by_field_name("body"),
        ),
        _ => (None, None),
    };
    let (Some(path), Some(list)) = (path, list) else {
        return false;
    };
    let name = text(path, source);
    let mut cursor = list.walk();
    let open = list.named_children(&mut cursor).any(|child| {
        let kind = child.kind();
        kind == "remaining_field_pattern" || kind == "base_field_initializer"
    });
    let named = name == "Self" || name == owner;
    named && !open
}
