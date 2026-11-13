use super::Measurement;
use crate::structural_lint::rule_identities::control_flow::ControlFlow;
use crate::structural_lint::rule_identities::kind::Kind;
use std::collections::BTreeSet;
use tree_sitter::Node;

fn named_value(node: Node<'_>) -> bool {
    match node.kind() {
        "identifier" | "self" => true,
        "parenthesized_expression" => {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .find(|child| !child.is_extra())
                .is_some_and(named_value)
        }
        "attribute" => node.child_by_field_name("object").is_some_and(named_value),
        "field_expression" => {
            let named_field = node
                .child_by_field_name("field")
                .is_some_and(|field| field.kind() == "field_identifier");
            named_field && node.child_by_field_name("value").is_some_and(named_value)
        }
        "scoped_identifier" => node.child_by_field_name("path").is_some_and(named_value),
        "crate" | "super" => true,
        _ => false,
    }
}
pub fn condition(test: Node<'_>, output: &mut Vec<Measurement>) {
    let invalid = !named_value(test);
    if invalid {
        let conditional = test.parent().unwrap_or(test);
        output.push(Measurement {
            kind: Kind::ControlFlow(ControlFlow::NamedIfCondition),
            line: conditional.start_position().row + 1,
            symbol: "if".into(),
            actual: 1,
        });
    }
}
// DECISION: D120
pub fn boolean_expression(node: Node<'_>, output: &mut Vec<Measurement>) {
    let nested = enclosing(node).is_some_and(connective);
    let measured = connective(node) && !nested;
    if measured {
        output.push(Measurement {
            kind: Kind::ControlFlow(ControlFlow::BooleanExpression),
            line: node.start_position().row + 1,
            symbol: "expression".into(),
            actual: operands(node),
        });
    }
}
// DECISION: D156
pub fn mixed_junction(node: Node<'_>) -> Option<usize> {
    let nested = enclosing(node).is_some_and(connective);
    let measured = connective(node) && !nested;
    let mut found = BTreeSet::new();
    junctions(node, &mut found);
    let mixed = measured && found.len() > 1;
    mixed.then_some(node.start_position().row + 1)
}
fn junctions<'a>(node: Node<'a>, found: &mut BTreeSet<&'static str>) {
    let inner = ungrouped(node);
    let operator = match inner.kind() {
        "let_chain" => Some("&&"),
        _ => inner
            .child_by_field_name("operator")
            .map(|operator| operator.kind())
            .filter(|_| connective(inner)),
    };
    let Some(operator) = operator else {
        return;
    };
    found.insert(operator);
    let mut cursor = inner.walk();
    let children: Vec<_> = inner.named_children(&mut cursor).collect();
    for child in children {
        junctions(child, found);
    }
}
fn connective(node: Node<'_>) -> bool {
    match node.kind() {
        "boolean_operator" | "let_chain" => true,
        "binary_expression" => node
            .child_by_field_name("operator")
            .is_some_and(|operator| matches!(operator.kind(), "&&" | "||")),
        _ => false,
    }
}
fn enclosing(node: Node<'_>) -> Option<Node<'_>> {
    let mut parent = node.parent()?;
    while parent.kind() == "parenthesized_expression" {
        parent = parent.parent()?;
    }
    Some(parent)
}
fn operands(node: Node<'_>) -> usize {
    let inner = ungrouped(node);
    match inner.kind() {
        "let_condition" => 0,
        _ if connective(inner) => {
            let mut cursor = inner.walk();
            inner
                .named_children(&mut cursor)
                .filter(|child| !child.is_extra())
                .map(operands)
                .sum()
        }
        _ => 1,
    }
}
fn ungrouped(node: Node<'_>) -> Node<'_> {
    let grouped = node.kind() == "parenthesized_expression";
    let mut cursor = node.walk();
    let inner = node
        .named_children(&mut cursor)
        .find(|child| !child.is_extra());
    match inner {
        Some(inner) if grouped => ungrouped(inner),
        _ => node,
    }
}
