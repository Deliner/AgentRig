use super::unhandled_error::{let_conditions, variant_pattern};
use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::context::in_test_context;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use tree_sitter::Node;

const SEARCHES: &[&str] = &["contains", "starts_with", "ends_with"];

pub(super) fn text_branch(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(function) = node
        .child_by_field_name("function")
        .filter(|function| function.kind() == "field_expression")
    else {
        return;
    };
    let Some(search) = function
        .child_by_field_name("field")
        .map(|field| text(field, source))
        .filter(|name| SEARCHES.contains(name))
    else {
        return;
    };
    let rendered = function
        .child_by_field_name("value")
        .and_then(|value| stringified(value, source).or_else(|| formatted(value, source)));
    let branched = rendered.is_some_and(|name| bound_by_error(node, name, source));
    let allowed = !branched || justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::ErrorTextBranch),
        line: node.start_position().row + 1,
        symbol: search.into(),
        actual: 1,
    });
}
fn stringified<'a>(value: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let empty = value
        .child_by_field_name("arguments")
        .is_some_and(|arguments| arguments.named_child_count() == 0);
    let function = value
        .child_by_field_name("function")
        .filter(|_| value.kind() == "call_expression" && empty)?;
    let named = function
        .child_by_field_name("field")
        .is_some_and(|field| text(field, source) == "to_string");
    function
        .child_by_field_name("value")
        .filter(|receiver| named && receiver.kind() == "identifier")
        .map(|receiver| text(receiver, source))
}
fn formatted<'a>(value: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let named = value.kind() == "macro_invocation"
        && value
            .child_by_field_name("macro")
            .is_some_and(|path| text(path, source) == "format");
    let tree = value
        .named_children(&mut value.walk())
        .find(|child| child.kind() == "token_tree")
        .filter(|_| named)?;
    let parts: Vec<_> = tree
        .named_children(&mut tree.walk())
        .filter(|child| !child.is_extra())
        .collect();
    match parts.as_slice() {
        [literal] => text(*literal, source)
            .strip_prefix("\"{")?
            .strip_suffix("}\"")
            .filter(|name| identifier(name)),
        [literal, argument] => {
            let positional = text(*literal, source) == "\"{}\"";
            (positional && argument.kind() == "identifier").then(|| text(*argument, source))
        }
        _ => None,
    }
}
fn identifier(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
}
enum Binding {
    Error,
    Other,
    Absent,
}
fn bound_by_error(call: Node<'_>, name: &str, source: Code<'_>) -> bool {
    let mut current = call;
    while let Some(parent) = current.parent() {
        let alternative = parent.child_by_field_name("alternative") == Some(current);
        let binding = if alternative {
            Binding::Absent
        } else {
            binder(parent, name, source)
        };
        match binding {
            Binding::Error => return true,
            Binding::Other => return false,
            Binding::Absent => current = parent,
        }
    }
    false
}
fn binder(parent: Node<'_>, name: &str, source: Code<'_>) -> Binding {
    match parent.kind() {
        "match_arm" | "if_expression" => {
            let bound = bound_patterns(parent)
                .into_iter()
                .any(|pattern| error_binding(pattern, name, source));
            if bound {
                Binding::Error
            } else {
                Binding::Absent
            }
        }
        "closure_expression" => closure_binding(parent, name, source),
        "function_item" => Binding::Other,
        _ => Binding::Absent,
    }
}
fn bound_patterns(parent: Node<'_>) -> Vec<Node<'_>> {
    match parent.kind() {
        "match_arm" => parent
            .child_by_field_name("pattern")
            .and_then(|pattern| pattern.named_child(0))
            .into_iter()
            .collect(),
        _ => parent
            .child_by_field_name("condition")
            .map(let_conditions)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|binding| binding.child_by_field_name("pattern"))
            .collect(),
    }
}
fn error_binding(pattern: Node<'_>, name: &str, source: Code<'_>) -> bool {
    let variant = pattern.child_by_field_name("type");
    let mut cursor = pattern.walk();
    let inner: Vec<_> = pattern
        .named_children(&mut cursor)
        .filter(|child| !child.is_extra() && Some(*child) != variant)
        .collect();
    let bound = matches!(inner.as_slice(), [binding] if binding.kind() == "identifier" && text(*binding, source) == name);
    bound && variant_pattern(pattern, "Err", source)
}
fn closure_binding(closure: Node<'_>, name: &str, source: Code<'_>) -> Binding {
    let Some(parameters) = closure.child_by_field_name("parameters") else {
        return Binding::Absent;
    };
    let mut cursor = parameters.walk();
    let names: Vec<_> = parameters
        .named_children(&mut cursor)
        .map(|parameter| {
            parameter
                .child_by_field_name("pattern")
                .unwrap_or(parameter)
        })
        .filter(|pattern| pattern.kind() == "identifier")
        .map(|pattern| text(pattern, source))
        .collect();
    let unbound = !names.contains(&name);
    if unbound {
        return Binding::Absent;
    }
    let mapped = names.len() == 1 && passed_to_map_err(closure, source);
    if mapped {
        Binding::Error
    } else {
        Binding::Other
    }
}
fn passed_to_map_err(closure: Node<'_>, source: Code<'_>) -> bool {
    closure
        .parent()
        .filter(|arguments| arguments.kind() == "arguments")
        .and_then(|arguments| arguments.parent())
        .and_then(|call| call.child_by_field_name("function"))
        .and_then(|function| function.child_by_field_name("field"))
        .is_some_and(|field| text(field, source) == "map_err")
}
