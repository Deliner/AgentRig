use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::context::{in_test_context, last_segment};
use crate::structural_lint::rule_identities::control_flow::ControlFlow;
use crate::structural_lint::rule_identities::kind::Kind;
use tree_sitter::Node;

// DECISION: D156
pub(super) fn comparison(binary: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let equality = binary
        .child_by_field_name("operator")
        .is_some_and(|operator| matches!(text(operator, source), "==" | "!="));
    let variant = ["left", "right"]
        .into_iter()
        .filter_map(|side| binary.child_by_field_name(side))
        .find(|operand| {
            operand.kind() == "scoped_identifier" && upper_camel(last_segment(*operand, source))
        });
    let Some(variant) = variant.filter(|_| equality) else {
        return;
    };
    report(binary, text(variant, source), source, output);
}
// DECISION: D156
pub(super) fn matched(invocation: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let named = invocation
        .child_by_field_name("macro")
        .is_some_and(|path| last_segment(path, source) == "matches");
    let mut cursor = invocation.walk();
    let Some(tree) = invocation
        .named_children(&mut cursor)
        .find(|child| child.kind() == "token_tree")
        .filter(|_| named)
    else {
        return;
    };
    let mut tokens = tree.walk();
    let pattern = tree
        .children(&mut tokens)
        .skip_while(|token| token.kind() != ",")
        .any(|token| names_variant(token, source));
    if pattern {
        report(invocation, "matches", source, output);
    }
}
fn report(node: Node<'_>, symbol: &str, source: Code<'_>, output: &mut Vec<Measurement>) {
    let allowed = in_test_context(node, source) || justified(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::ControlFlow(ControlFlow::VariantEquality),
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn names_variant(token: Node<'_>, source: Code<'_>) -> bool {
    let segment = token.kind() == "identifier"
        && token
            .prev_sibling()
            .is_some_and(|previous| previous.kind() == "::");
    if segment {
        return upper_camel(text(token, source));
    }
    let mut cursor = token.walk();
    let children: Vec<_> = token.children(&mut cursor).collect();
    children
        .into_iter()
        .any(|child| names_variant(child, source))
}
fn upper_camel(name: &str) -> bool {
    let capital = name
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_uppercase());
    let constant = name.len() > 1 && !name.chars().any(|letter| letter.is_ascii_lowercase());
    capital && !constant && !name.contains('_')
}
