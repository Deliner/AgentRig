use super::{Code, Measurement, text};
use crate::structural_lint::rule_identities::control_flow::ControlFlow;
use crate::structural_lint::rule_identities::kind::Kind;
use tree_sitter::Node;

// DECISION: D061
pub enum Nest {
    Increment,
    Boundary,
    Plain,
}
pub fn nesting(
    node: Node<'_>,
    source: Code<'_>,
    classify: fn(Node<'_>) -> Nest,
    output: &mut Vec<Measurement>,
) {
    let has_body = node.child_by_field_name("body").is_some();
    if has_body {
        let symbol = node
            .child_by_field_name("name")
            .map(|name| text(name, source))
            .unwrap_or("<anonymous>");
        let mut max = 0;
        descend(node, classify, 0, &mut max);
        output.push(Measurement {
            kind: Kind::ControlFlow(ControlFlow::NestingDepth),
            line: node.start_position().row + 1,
            symbol: symbol.into(),
            actual: max,
        });
    }
}
fn descend(node: Node<'_>, classify: fn(Node<'_>) -> Nest, depth: usize, max: &mut usize) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match classify(child) {
            Nest::Boundary => {}
            Nest::Increment => {
                let inner = depth + 1;
                *max = (*max).max(inner);
                descend(child, classify, inner, max);
            }
            Nest::Plain => descend(child, classify, depth, max),
        }
    }
}
// DECISION: D063
pub enum Branch {
    Boundary,
    Decision,
    Plain,
}
pub fn branching(
    node: Node<'_>,
    source: Code<'_>,
    classify: fn(Node<'_>) -> Branch,
    output: &mut Vec<Measurement>,
) {
    let has_body = node.child_by_field_name("body").is_some();
    if has_body {
        let symbol = node
            .child_by_field_name("name")
            .map(|name| text(name, source))
            .unwrap_or("<anonymous>");
        let mut total = 0;
        accumulate(node, classify, &mut total);
        output.push(Measurement {
            kind: Kind::ControlFlow(ControlFlow::BranchBudget),
            line: node.start_position().row + 1,
            symbol: symbol.into(),
            actual: total,
        });
    }
}
fn accumulate(node: Node<'_>, classify: fn(Node<'_>) -> Branch, total: &mut usize) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match classify(child) {
            Branch::Boundary => {}
            Branch::Decision => {
                *total += 1;
                accumulate(child, classify, total);
            }
            Branch::Plain => accumulate(child, classify, total),
        }
    }
}
