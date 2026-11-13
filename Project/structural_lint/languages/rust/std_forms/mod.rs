mod dotted_extension;
mod epsilon_tolerance;
mod superseded_form;
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::exemptions::in_test_context;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use tree_sitter::Node;

pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    superseded_form::inspect(node, source, output);
    let Some((reference, symbol)) = form(node, source) else {
        return;
    };
    let allowed = justified(node, source) || in_test_context(node, source);
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
fn form(node: Node<'_>, source: Code<'_>) -> Option<(Reference, &'static str)> {
    match node.kind() {
        "binary_expression" => epsilon_tolerance::inspect(node, source)
            .map(|symbol| (Reference::EpsilonTolerance, symbol)),
        "call_expression" => dotted_extension::inspect(node, source)
            .map(|symbol| (Reference::DottedExtension, symbol)),
        _ => None,
    }
}
fn method<'t, 's>(call: Node<'t>, source: Code<'s>) -> Option<(&'s str, Node<'t>)> {
    let invoked = call.kind() == "call_expression";
    let function = call.child_by_field_name("function").filter(|_| invoked)?;
    let field = function.child_by_field_name("field")?;
    Some((text(field, source), function.child_by_field_name("value")?))
}
fn sole_argument(call: Node<'_>) -> Option<Node<'_>> {
    let arguments = call.child_by_field_name("arguments")?;
    let mut cursor = arguments.walk();
    let mut values = arguments
        .named_children(&mut cursor)
        .filter(|argument| !argument.is_extra());
    let first = values.next()?;
    let single = values.next().is_none();
    single.then_some(first)
}
