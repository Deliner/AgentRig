use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::exemptions::last_segment;
use crate::structural_lint::rule_identities::control_flow::ControlFlow;
use crate::structural_lint::rule_identities::kind::Kind;
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::Node;

// DECISION: D206
const LEAST_VARIANTS: usize = 3;

// DECISION: D206
pub(super) fn inspect(expression: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(symbol) = classification(expression, source) else {
        return;
    };
    let allowed = justified(expression, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::ControlFlow(ControlFlow::RepeatedClassification),
        line: expression.start_position().row + 1,
        symbol,
        actual: 1,
    });
}
fn classification(expression: Node<'_>, source: Code<'_>) -> Option<String> {
    let block = expression.child_by_field_name("body")?;
    let mut cursor = block.walk();
    let mut sides: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for arm in block
        .named_children(&mut cursor)
        .filter(|arm| arm.kind() == "match_arm")
    {
        let pattern = arm.child_by_field_name("pattern")?;
        let unguarded = pattern.child_by_field_name("condition").is_none();
        let value = arm
            .child_by_field_name("value")
            .filter(|value| unguarded && value.kind() == "boolean_literal")?;
        let paths = variants(pattern.named_child(0)?, source)?;
        sides.entry(text(value, source)).or_default().extend(paths);
    }
    let named: usize = sides.values().map(BTreeSet::len).sum();
    let classified = sides.len() == 2 && named >= LEAST_VARIANTS;
    classified.then(|| {
        let arms: Vec<_> = sides
            .iter()
            .map(|(result, paths)| {
                let alternatives: Vec<_> = paths.iter().map(String::as_str).collect();
                format!("{} => {result}", alternatives.join(" | "))
            })
            .collect();
        arms.join(", ")
    })
}
fn variants(pattern: Node<'_>, source: Code<'_>) -> Option<Vec<String>> {
    match pattern.kind() {
        "scoped_identifier" => {
            let owner = last_segment(pattern.child_by_field_name("path")?, source);
            let name = text(pattern.child_by_field_name("name")?, source);
            Some(vec![format!("{owner}::{name}")])
        }
        "or_pattern" => {
            let mut cursor = pattern.walk();
            let alternatives: Option<Vec<_>> = pattern
                .named_children(&mut cursor)
                .map(|alternative| variants(alternative, source))
                .collect();
            alternatives.map(|nested| nested.concat())
        }
        _ => None,
    }
}
