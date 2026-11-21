use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, text};
use crate::structural_lint::languages::rust::exemptions::{in_test_context, last_segment};
use crate::structural_lint::rule_identities::{Kind, Reference};
use tree_sitter::Node;

const ERROR_ARMS: &[&str] = &["Err(_)", "Err(..)"];

// DECISION: D097
pub(super) fn match_arm(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let swallowed = node
        .child_by_field_name("pattern")
        .is_some_and(|pattern| error_pattern(pattern, source))
        && node
            .child_by_field_name("value")
            .is_some_and(|value| answers(value, source));
    if swallowed {
        report_verdict(node, source, output);
    }
}
pub(super) fn if_let(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let swallowed = node
        .child_by_field_name("condition")
        .and_then(|condition| condition.child_by_field_name("pattern"))
        .is_some_and(|pattern| error_pattern(pattern, source))
        && node
            .child_by_field_name("consequence")
            .is_some_and(|block| answers(block, source));
    if swallowed {
        report_verdict(node, source, output);
    }
}
pub(super) fn bool_result(annotation: Node<'_>, source: Code<'_>) -> bool {
    let generic = annotation.kind() == "generic_type";
    let other = !generic;
    if other {
        return false;
    }
    let result = annotation
        .child_by_field_name("type")
        .is_some_and(|path| last_segment(path, source) == "Result");
    let flag = annotation
        .child_by_field_name("type_arguments")
        .and_then(|arguments| arguments.named_child(0))
        .is_some_and(|first| first.kind() == "primitive_type" && text(first, source) == "bool");
    result && flag
}
pub(super) fn ok_argument<'a>(call: Node<'a>, source: Code<'_>) -> Option<Node<'a>> {
    let constructor = call
        .child_by_field_name("function")
        .is_some_and(|function| function.kind() == "identifier" && text(function, source) == "Ok");
    let arguments = constructor
        .then(|| call.child_by_field_name("arguments"))
        .flatten()?;
    let mut cursor = arguments.walk();
    let values: Vec<_> = arguments
        .named_children(&mut cursor)
        .filter(|child| !child.is_extra())
        .collect();
    match values.as_slice() {
        [value] => Some(*value),
        _ => None,
    }
}
fn report_verdict(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(function) = verdict_function(node, source) else {
        return;
    };
    let allowed = in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::SilentError),
        line: node.start_position().row + 1,
        symbol: function.into(),
        actual: 1,
    });
}
fn error_pattern(pattern: Node<'_>, source: Code<'_>) -> bool {
    let compact: String = text(pattern, source).split_whitespace().collect();
    ERROR_ARMS.contains(&compact.as_str())
}
fn answers(node: Node<'_>, source: Code<'_>) -> bool {
    match node.kind() {
        "block" | "expression_statement" | "return_expression" => {
            let mut cursor = node.walk();
            let mut inner = node
                .named_children(&mut cursor)
                .filter(|child| !child.is_extra());
            let first = inner.next();
            let lone = inner.next().is_none();
            lone && first.is_some_and(|child| answers(child, source))
        }
        "call_expression" => verdict_call(node, source),
        _ => false,
    }
}
fn verdict_call(node: Node<'_>, source: Code<'_>) -> bool {
    ok_argument(node, source).is_some_and(|value| value.kind() == "boolean_literal")
}
fn verdict_function<'a>(node: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let mut current = node.parent()?;
    loop {
        let boundary = current.kind() == "closure_expression";
        if boundary {
            return None;
        }
        let function = current.kind() == "function_item";
        if function {
            break;
        }
        current = current.parent()?;
    }
    let returns_verdict = current
        .child_by_field_name("return_type")
        .is_some_and(|annotation| bool_result(annotation, source));
    let name = current.child_by_field_name("name")?;
    returns_verdict.then_some(text(name, source))
}
