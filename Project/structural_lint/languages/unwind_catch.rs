use super::test_context::{in_test_context, last_segment};
use super::{Measurement, justified, text};
use crate::structural_lint::rules::kind::Kind;
use tree_sitter::Node;

// DECISION: D115
pub(super) fn inspect(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let Some(symbol) = catch_site(function, source) else {
        return;
    };
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::UnwindJustification,
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn catch_site<'a>(function: Node<'_>, source: &'a str) -> Option<&'a str> {
    match function.kind() {
        "field_expression" => {
            let name = text(function.child_by_field_name("field")?, source);
            (name == "catch_unwind").then_some(name)
        }
        "identifier" | "scoped_identifier" => {
            let name = last_segment(function, source);
            matches!(name, "catch_unwind" | "AssertUnwindSafe").then_some(name)
        }
        _ => None,
    }
}
