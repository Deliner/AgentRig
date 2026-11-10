use crate::structural_lint::languages::measure::{Measurement, text};
use crate::structural_lint::languages::rust::context::{in_test_context, justified_item};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use std::collections::BTreeSet;
use tree_sitter::Node;

// DECISION: D156
pub(super) fn parameters(function: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(parameters) = function.child_by_field_name("parameters") else {
        return;
    };
    let mut cursor = parameters.walk();
    let types: Vec<_> = parameters
        .named_children(&mut cursor)
        .filter(|parameter| parameter.kind() == "parameter")
        .filter(|parameter| {
            parameter
                .child_by_field_name("pattern")
                .is_none_or(|pattern| pattern.kind() != "self")
        })
        .filter_map(|parameter| parameter.child_by_field_name("type"))
        .collect();
    let exempt = trait_method(function) || !repeated(&types, source);
    if exempt {
        return;
    }
    report(function, source, Signature::SameTypeParameters, output);
}
// DECISION: D156
pub(super) fn positions(item: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(body) = item
        .child_by_field_name("body")
        .filter(|body| body.kind() == "ordered_field_declaration_list")
    else {
        return;
    };
    let mut cursor = body.walk();
    let types: Vec<_> = body.children_by_field_name("type", &mut cursor).collect();
    let unique = !repeated(&types, source);
    if unique {
        return;
    }
    report(item, source, Signature::SameTypePositions, output);
}
fn report(item: Node<'_>, source: &str, kind: Signature, output: &mut Vec<Measurement>) {
    let allowed = in_test_context(item, source) || justified_item(item, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Signature(kind),
        line: item.start_position().row + 1,
        symbol: item
            .child_by_field_name("name")
            .map_or("", |name| text(name, source))
            .into(),
        actual: 1,
    });
}
fn repeated(types: &[Node<'_>], source: &str) -> bool {
    let mut seen = BTreeSet::new();
    types
        .iter()
        .any(|annotation| !seen.insert(tokens(*annotation, source)))
}
fn tokens(node: Node<'_>, source: &str) -> String {
    text(node, source).split_whitespace().collect()
}
fn trait_method(function: Node<'_>) -> bool {
    function
        .parent()
        .and_then(|list| list.parent())
        .is_some_and(|owner| {
            owner.kind() == "impl_item" && owner.child_by_field_name("trait").is_some()
        })
}
