use crate::structural_lint::languages::measure::{Measurement, justified, text};
use crate::structural_lint::languages::rust::context::in_test_context;
use crate::structural_lint::rules::control_flow::ControlFlow;
use crate::structural_lint::rules::kind::Kind;
use std::collections::BTreeMap;
use tree_sitter::Node;

// DECISION: D156
pub(super) fn chain(branch: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let continuation = branch
        .parent()
        .is_some_and(|parent| parent.kind() == "else_clause");
    if continuation {
        return;
    }
    let mut subjects = Vec::new();
    let mut current = Some(branch);
    while let Some(link) = current {
        subjects.push(
            link.child_by_field_name("condition")
                .and_then(|condition| bounded(condition, source)),
        );
        current = link
            .child_by_field_name("alternative")
            .and_then(|alternative| alternative.named_child(0))
            .filter(|next| next.kind() == "if_expression");
    }
    let first = subjects.first().cloned().flatten();
    let ranged =
        subjects.len() > 1 && first.is_some() && subjects.iter().all(|subject| *subject == first);
    if ranged {
        report(branch, source, output);
    }
}
// DECISION: D156
pub(super) fn guards(expression: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(block) = expression.child_by_field_name("body") else {
        return;
    };
    let mut cursor = block.walk();
    let mut counts = BTreeMap::new();
    let subjects = block
        .named_children(&mut cursor)
        .filter_map(|arm| arm.child_by_field_name("pattern"))
        .filter_map(|pattern| pattern.child_by_field_name("condition"))
        .filter_map(|condition| bounded(condition, source));
    for subject in subjects {
        *counts.entry(subject).or_insert(0_usize) += 1;
    }
    let ranged = counts.values().any(|count| *count > 1);
    if ranged {
        report(expression, source, output);
    }
}
fn report(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let allowed = in_test_context(node, source) || justified(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::ControlFlow(ControlFlow::RangeChain),
        line: node.start_position().row + 1,
        symbol: node.kind().into(),
        actual: 1,
    });
}
fn bounded(condition: Node<'_>, source: &str) -> Option<String> {
    let ordering = condition.kind() == "binary_expression"
        && condition
            .child_by_field_name("operator")
            .is_some_and(|operator| matches!(text(operator, source), "<" | "<=" | ">" | ">="));
    let left = condition.child_by_field_name("left").filter(|_| ordering)?;
    let right = condition.child_by_field_name("right")?;
    let subject = match (literal(left, source), literal(right, source)) {
        (true, false) => right,
        (false, true) => left,
        _ => return None,
    };
    Some(text(subject, source).split_whitespace().collect())
}
fn literal(node: Node<'_>, source: &str) -> bool {
    match node.kind() {
        "integer_literal" => true,
        "unary_expression" => {
            let negative = text(node, source).starts_with('-');
            negative
                && node
                    .named_child(0)
                    .is_some_and(|inner| literal(inner, source))
        }
        _ => false,
    }
}
