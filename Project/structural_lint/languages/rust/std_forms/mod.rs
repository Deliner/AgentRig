mod call_syntax;
mod dotted_extension;
mod eager_fallback;
mod epsilon_tolerance;
mod flag_filter;
mod indexed_array;
mod max_fallback;
mod superseded_form;
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement};
use crate::structural_lint::languages::rust::exemptions::in_test_context;
use crate::structural_lint::rule_identities::{Kind, Reference};
use tree_sitter::Node;

pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    superseded_form::inspect(node, source, output);
    indexed_array::inspect(node, output);
    flag_filter::inspect(node, source, output);
    let Some((reference, symbol)) = form(node, source) else {
        return;
    };
    let allowed = in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(reference),
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
pub(super) fn call(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    eager_fallback::inspect(node, source, output);
}
fn form(node: Node<'_>, source: Code<'_>) -> Option<(Reference, &'static str)> {
    match node.kind() {
        "binary_expression" => epsilon_tolerance::inspect(node, source)
            .map(|symbol| (Reference::EpsilonTolerance, symbol))
            .or_else(|| {
                max_fallback::inspect(node, source).map(|symbol| (Reference::MaxFallback, symbol))
            }),
        "call_expression" => dotted_extension::inspect(node, source)
            .map(|symbol| (Reference::DottedExtension, symbol)),
        _ => None,
    }
}
