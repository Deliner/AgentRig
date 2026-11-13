mod dotted_extension;
mod epsilon_tolerance;
mod front_removal;
mod partial_order;
mod remainder_sign;
mod superseded_form;
mod unstable_output;
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::exemptions::in_test_context;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use tree_sitter::Node;

pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    superseded_form::inspect(node, source, output);
    for (reference, form) in forms(node, source) {
        let Some(symbol) = form else {
            continue;
        };
        let allowed = justified(node, source) || in_test_context(node, source);
        if allowed {
            continue;
        }
        output.push(Measurement {
            kind: Kind::Reference(reference),
            line: node.start_position().row + 1,
            symbol: symbol.into(),
            actual: 1,
        });
    }
}
fn forms(node: Node<'_>, source: Code<'_>) -> Vec<(Reference, Option<&'static str>)> {
    match node.kind() {
        "binary_expression" => vec![
            (
                Reference::RemainderSign,
                remainder_sign::inspect(node, source),
            ),
            (
                Reference::EpsilonTolerance,
                epsilon_tolerance::inspect(node, source),
            ),
        ],
        "call_expression" => vec![
            (
                Reference::PartialOrderFallback,
                partial_order::inspect(node, source),
            ),
            (
                Reference::UnstableValue,
                unstable_output::named_type(node, source),
            ),
            (Reference::UnstableSort, unstable_output::sort(node, source)),
            (
                Reference::FrontRemoval,
                front_removal::inspect(node, source),
            ),
            (
                Reference::DottedExtension,
                dotted_extension::inspect(node, source),
            ),
        ],
        "identifier" => vec![(
            Reference::UnstableValue,
            unstable_output::hasher(node, source),
        )],
        _ => Vec::new(),
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
fn bare(mut operand: Node<'_>) -> Node<'_> {
    while let Some(inner) = operand
        .named_child(0)
        .filter(|_| operand.kind() == "parenthesized_expression")
    {
        operand = inner;
    }
    operand
}
