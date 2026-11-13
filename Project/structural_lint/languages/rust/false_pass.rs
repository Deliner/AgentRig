use super::exemptions::{attribute_path, cfg_excludes, in_test_context, last_segment};
use crate::structural_lint::languages::neutral_metrics::{
    Code, Measurement, descendants, justified, text,
};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use tree_sitter::Node;

// DECISION: D169
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    match node.kind() {
        "attribute_item" => {
            untargeted_panic(node, source, output);
            test_substitution(node, source, output);
        }
        "inner_attribute_item" => test_substitution(node, source, output),
        _ => {}
    }
}
fn untargeted_panic(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(path) = attribute_path(node) else {
        return;
    };
    let other = last_segment(path, source) != "should_panic";
    if other {
        return;
    }
    let Some(attribute) = path.parent() else {
        return;
    };
    let valued = attribute.child_by_field_name("value").is_some();
    let expected = attribute
        .child_by_field_name("arguments")
        .is_some_and(|arguments| names_expected(arguments, source));
    let targeted = valued || expected;
    let allowed = targeted || justified(node, source);
    if allowed {
        return;
    }
    report(node, Reference::UntargetedPanic, "should_panic", output);
}
fn names_expected(arguments: Node<'_>, source: Code<'_>) -> bool {
    descendants(arguments)
        .into_iter()
        .any(|token| token.kind() == "identifier" && text(token, source) == "expected")
}
fn test_substitution(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let item = node
        .parent()
        .is_some_and(|parent| matches!(parent.kind(), "source_file" | "declaration_list"));
    let substituted = item && cfg_excludes(node, source, "test");
    let other = !substituted;
    if other {
        return;
    }
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    report(node, Reference::TestSubstitution, "cfg", output);
}
fn report(node: Node<'_>, reference: Reference, symbol: &str, output: &mut Vec<Measurement>) {
    output.push(Measurement {
        kind: Kind::Reference(reference),
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
