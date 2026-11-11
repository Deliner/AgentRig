use crate::structural_lint::languages::measure::{Code, Measurement, descendants, justified, text};
use crate::structural_lint::languages::rust::context::{in_test_context, last_segment};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::ownership::Ownership;
use tree_sitter::Node;

const SETTINGS: &[(&str, &str)] = &[
    ("env", "set_var"),
    ("env", "remove_var"),
    ("env", "set_current_dir"),
    ("panic", "set_hook"),
    ("log", "set_logger"),
    ("subscriber", "set_global_default"),
];

pub(super) fn inspect(call: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(function) = call.child_by_field_name("function") else {
        return;
    };
    let finding = match function.kind() {
        "field_expression" => method(call, source).filter(|_| !in_test_context(call, source)),
        "scoped_identifier" => setting(call, source),
        _ => None,
    };
    let Some((kind, symbol)) = finding.filter(|_| !justified(call, source)) else {
        return;
    };
    output.push(Measurement {
        kind: Kind::Ownership(kind),
        line: call.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn method<'a>(call: Node<'_>, source: Code<'a>) -> Option<(Ownership, &'a str)> {
    let function = call.child_by_field_name("function")?;
    let name = text(function.child_by_field_name("field")?, source);
    let receiver = text(function.child_by_field_name("value")?, source);
    let arguments = call.child_by_field_name("arguments")?;
    let mut cursor = arguments.walk();
    let mut values = arguments
        .named_children(&mut cursor)
        .filter(|value| !value.is_extra());
    match name {
        "store" => {
            let stored = values.next()?;
            let reread = descendants(stored)
                .into_iter()
                .any(|node| loaded(node, receiver, source));
            reread.then_some((Ownership::LoadThenStore, receiver))
        }
        "wait" | "wait_timeout" => {
            let unlooped = values.next().is_some() && !looped(call);
            unlooped.then_some((Ownership::UnloopedWait, name))
        }
        _ => None,
    }
}
fn loaded(node: Node<'_>, receiver: &str, source: Code<'_>) -> bool {
    let Some(function) = node.child_by_field_name("function").filter(|function| {
        node.kind() == "call_expression" && function.kind() == "field_expression"
    }) else {
        return false;
    };
    let load = function
        .child_by_field_name("field")
        .is_some_and(|field| text(field, source) == "load");
    load && function
        .child_by_field_name("value")
        .is_some_and(|value| text(value, source) == receiver)
}
fn looped(call: Node<'_>) -> bool {
    let mut current = call;
    while let Some(parent) = current.parent() {
        match parent.kind() {
            "while_expression" | "loop_expression" => return true,
            "function_item" | "closure_expression" => return false,
            _ => current = parent,
        }
    }
    false
}
fn setting<'a>(call: Node<'_>, source: Code<'a>) -> Option<(Ownership, &'a str)> {
    let function = call.child_by_field_name("function")?;
    let name = text(function.child_by_field_name("name")?, source);
    let module = last_segment(function.child_by_field_name("path")?, source);
    let listed = SETTINGS.contains(&(module, name));
    let main = enclosing_function(call)
        .and_then(|item| item.child_by_field_name("name"))
        .is_some_and(|item| text(item, source) == "main");
    (listed && !main).then_some((Ownership::ProcessSetting, name))
}
fn enclosing_function(node: Node<'_>) -> Option<Node<'_>> {
    let mut current = node.parent();
    while let Some(parent) = current {
        let function = parent.kind() == "function_item";
        if function {
            return Some(parent);
        }
        current = parent.parent();
    }
    None
}
