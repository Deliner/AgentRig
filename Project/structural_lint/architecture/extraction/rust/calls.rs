// DECISION: D125
use super::{Source, compact};
use crate::structural_lint::architecture::extraction::references::PropagatedCall;
use tree_sitter::Node;

pub(super) fn inspect(source: &mut Source<'_>, call: Node<'_>) {
    let method = call
        .child_by_field_name("function")
        .filter(|function| function.kind() == "field_expression")
        .and_then(|function| function.child_by_field_name("field"));
    let Some(method) = method else {
        return;
    };
    let bare = call
        .child_by_field_name("arguments")
        .is_some_and(|arguments| arguments.named_child_count() == 0);
    let propagated = bare && propagates(source, call);
    if propagated {
        push(source, method);
    }
}

pub(super) fn token(source: &mut Source<'_>, tokens: &[Node<'_>], index: usize) {
    let Some([dot, method, arguments, rest @ ..]) = tokens.get(index..) else {
        return;
    };
    let called = dot.kind() == "." && method.kind() == "identifier";
    let bare = called && compact(source.text(*arguments)) == "()";
    let propagated = bare && token_propagates(source, rest);
    if propagated {
        push(source, *method);
    }
}

fn push(source: &mut Source<'_>, method: Node<'_>) {
    let call = PropagatedCall {
        method: source.text(method).to_owned(),
        line: method.start_position().row + 1,
    };
    source.output.propagated_calls.push(call);
}

fn token_propagates(source: &Source<'_>, rest: &[Node<'_>]) -> bool {
    let tried = rest.first().is_some_and(|token| token.kind() == "?");
    let converted = match rest {
        [dot, converter, arguments, ..] => {
            let mapped = dot.kind() == "." && source.text(*converter) == "map_err";
            mapped && compact(source.text(*arguments)) == "(Into::into)"
        }
        _ => false,
    };
    tried || converted
}

fn propagates(source: &Source<'_>, call: Node<'_>) -> bool {
    let Some(parent) = call.parent() else {
        return false;
    };
    let tried = parent.kind() == "try_expression";
    if tried {
        return true;
    }
    let receiver =
        parent.kind() == "field_expression" && parent.child_by_field_name("value") == Some(call);
    let converter = parent
        .child_by_field_name("field")
        .is_some_and(|field| source.text(field) == "map_err");
    let arguments = parent
        .parent()
        .filter(|outer| receiver && converter && outer.kind() == "call_expression")
        .and_then(|outer| outer.child_by_field_name("arguments"));
    arguments.is_some_and(|arguments| {
        let mut cursor = arguments.walk();
        let values: Vec<_> = arguments.named_children(&mut cursor).collect();
        matches!(values.as_slice(), [value] if compact(source.text(*value)) == "Into::into")
    })
}
