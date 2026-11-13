use crate::structural_lint::languages::neutral_metrics::{
    Code, Measurement, descendants, justified, text,
};
use crate::structural_lint::languages::rust::exemptions::{
    attribute_path, attributed, cfg_excludes, in_test_context, justified_item, last_segment,
};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use std::iter::successors;
use tree_sitter::Node;

// DECISION: D169
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    match node.kind() {
        "attribute_item" => {
            untargeted_panic(node, source, output);
            test_substitution(node, source, output);
        }
        "inner_attribute_item" => test_substitution(node, source, output),
        "function_item" => conditional_check(node, source, output),
        _ => {}
    }
}
fn conditional_check(function: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let test = attributed(function, source, |attribute, source| {
        attribute_path(attribute).is_some_and(|path| last_segment(path, source) == "test")
    });
    let Some(body) = function.child_by_field_name("body").filter(|_| test) else {
        return;
    };
    let checks: Vec<_> = descendants(body)
        .into_iter()
        .filter(|node| assertion(*node, source))
        .collect();
    let branched = checks.iter().all(|check| {
        successors(Some(*check), Node::parent)
            .take_while(|node| *node != function)
            .any(open_branch)
    });
    let conditional = !checks.is_empty() && branched;
    let allowed = !conditional || justified_item(function, source);
    if allowed {
        return;
    }
    let name = function
        .child_by_field_name("name")
        .map_or("test", |name| text(name, source));
    report(function, Reference::ConditionalCheck, name, output);
}
fn assertion(node: Node<'_>, source: Code<'_>) -> bool {
    node.kind() == "macro_invocation"
        && node.child_by_field_name("macro").is_some_and(|name| {
            matches!(
                last_segment(name, source),
                "assert" | "assert_eq" | "assert_ne"
            )
        })
}
fn open_branch(node: Node<'_>) -> bool {
    node.parent().is_some_and(|parent| {
        parent.kind() == "if_expression"
            && parent.child_by_field_name("consequence") == Some(node)
            && open(parent)
    })
}
fn open(branch: Node<'_>) -> bool {
    successors(Some(branch), |current| {
        let alternative = current.child_by_field_name("alternative")?;
        alternative
            .named_children(&mut alternative.walk())
            .find(|next| next.kind() == "if_expression")
    })
    .last()
    .is_some_and(|last| last.child_by_field_name("alternative").is_none())
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
