use crate::structural_lint::languages::measure::{Code, Measurement, text};
use crate::structural_lint::languages::rust::context::{
    in_test_context, justified_item, trait_impl_member,
};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use std::collections::BTreeSet;
use tree_sitter::Node;

// DECISION: D156
pub(super) fn parameters(function: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
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
    let exempt = trait_impl_member(function) || !repeated(&types, source);
    if exempt {
        return;
    }
    report(function, source, Signature::SameTypeParameters, output);
}
// DECISION: D156
pub(super) fn positions(item: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(body) = item
        .child_by_field_name("body")
        .filter(|body| body.kind() == "ordered_field_declaration_list")
    else {
        return;
    };
    let mut cursor = body.walk();
    let types: Vec<_> = body.children_by_field_name("type", &mut cursor).collect();
    let duplicated = repeated(&types, source);
    if duplicated {
        report(item, source, Signature::SameTypePositions, output);
    }
    let flagged = types.len() > 1
        && types
            .iter()
            .any(|annotation| text(*annotation, source) == "bool");
    if flagged {
        report(item, source, Signature::PositionalFlag, output);
    }
}
fn report(item: Node<'_>, source: Code<'_>, kind: Signature, output: &mut Vec<Measurement>) {
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
fn repeated(types: &[Node<'_>], source: Code<'_>) -> bool {
    let mut seen = BTreeSet::new();
    types
        .iter()
        .any(|annotation| !seen.insert(tokens(*annotation, source)))
}
fn tokens(node: Node<'_>, source: Code<'_>) -> String {
    text(node, source).split_whitespace().collect()
}
