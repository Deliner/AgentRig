use super::call_syntax::method_receiver;
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, text, values};
use crate::structural_lint::languages::rust::exemptions::in_test_context;
use crate::structural_lint::rule_identities::{Kind, Reference};
use tree_sitter::Node;

// DECISION: D097
// DECISION: D165
// DECISION: D207
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let Some(field) = function
        .child_by_field_name("field")
        .filter(|field| field.kind() == "field_identifier")
    else {
        return;
    };
    let Some(receiver) = function.child_by_field_name("value") else {
        return;
    };
    let Some(arguments) = node.child_by_field_name("arguments") else {
        return;
    };
    let name = text(field, source);
    let dropped = drops_error(name, &values(arguments), receiver, source);
    let allowed = !dropped || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::SilentError),
        line: node.start_position().row + 1,
        symbol: name.into(),
        actual: 1,
    });
}
// DECISION: D207
fn drops_error(name: &str, arguments: &[Node<'_>], receiver: Node<'_>, source: Code<'_>) -> bool {
    match (name, arguments) {
        ("flatten", []) => {
            read_dir_rooted(receiver, source) || method_receiver(receiver, "ok", source).is_some()
        }
        ("map_while", [argument]) => {
            result_ok(*argument, source) && read_dir_rooted(receiver, source)
        }
        ("filter_map" | "flat_map", [argument]) => {
            let clippy = lines_call(receiver, source) && plain_ok(*argument, source);
            result_ok(*argument, source) && !clippy
        }
        ("ok" | "unwrap_or_default", []) | ("unwrap_or" | "unwrap_or_else", [_]) => {
            method_receiver(receiver, "join", source).is_some()
        }
        _ => false,
    }
}
fn result_ok(argument: Node<'_>, source: Code<'_>) -> bool {
    ok_path(argument, source) || ok_receiver(argument, source).is_some()
}
fn ok_path(argument: Node<'_>, source: Code<'_>) -> bool {
    argument.kind() == "scoped_identifier" && text(argument, source) == "Result::ok"
}
fn ok_receiver<'a>(argument: Node<'a>, source: Code<'_>) -> Option<Node<'a>> {
    let closure = argument.kind() == "closure_expression";
    let parameters = closure
        .then(|| argument.child_by_field_name("parameters"))
        .flatten()?;
    let body = argument
        .child_by_field_name("body")
        .filter(|_| parameters.named_child_count() == 1)?;
    let empty = body
        .child_by_field_name("arguments")
        .is_some_and(|arguments| arguments.named_child_count() == 0);
    let function = body
        .child_by_field_name("function")
        .filter(|_| body.kind() == "call_expression" && empty)?;
    let ok = function
        .child_by_field_name("field")
        .is_some_and(|field| text(field, source) == "ok");
    ok.then(|| function.child_by_field_name("value")).flatten()
}
fn lines_call(receiver: Node<'_>, source: Code<'_>) -> bool {
    receiver.kind() == "call_expression"
        && receiver
            .child_by_field_name("function")
            .is_some_and(|function| {
                function.kind() == "field_expression"
                    && callee_name(function, source) == Some("lines")
            })
}
fn plain_ok(argument: Node<'_>, source: Code<'_>) -> bool {
    let parameter = argument
        .child_by_field_name("parameters")
        .and_then(|parameters| parameters.named_child(0));
    let itself =
        ok_receiver(argument, source)
            .zip(parameter)
            .is_some_and(|(receiver, parameter)| {
                receiver.kind() == "identifier" && text(receiver, source) == text(parameter, source)
            });
    ok_path(argument, source) || itself
}
fn read_dir_rooted(receiver: Node<'_>, source: Code<'_>) -> bool {
    let mut current = receiver;
    while let Some(inner) = unwrapped(current, source) {
        current = inner;
    }
    let call = current.kind() == "call_expression";
    call && current
        .child_by_field_name("function")
        .is_some_and(|function| callee_name(function, source) == Some("read_dir"))
}
fn unwrapped<'a>(node: Node<'a>, source: Code<'_>) -> Option<Node<'a>> {
    let propagated = node.kind() == "try_expression";
    if propagated {
        return node.named_child(0);
    }
    let call = node.kind() == "call_expression";
    let other = !call;
    if other {
        return None;
    }
    let function = node.child_by_field_name("function")?;
    let adapter =
        function.kind() == "field_expression" && callee_name(function, source) == Some("into_iter");
    if adapter {
        return function.child_by_field_name("value");
    }
    None
}
fn callee_name<'a>(function: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    match function.kind() {
        "identifier" => Some(text(function, source)),
        "scoped_identifier" => function
            .child_by_field_name("name")
            .map(|name| text(name, source)),
        "field_expression" => function
            .child_by_field_name("field")
            .map(|field| text(field, source)),
        _ => None,
    }
}
