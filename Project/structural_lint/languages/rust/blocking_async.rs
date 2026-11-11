use super::context::async_code::in_async_context;
use super::context::{in_test_context, last_segment};
use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use tree_sitter::Node;

// DECISION: D118
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let Some(symbol) = blocking(function, source) else {
        return;
    };
    let allowed = justified(node, source) || in_test_context(node, source);
    let reported = !allowed && in_async_context(node);
    if reported {
        output.push(Measurement {
            kind: Kind::Reference(Reference::BlockingInAsync),
            line: node.start_position().row + 1,
            symbol: symbol.into(),
            actual: 1,
        });
    }
}
fn blocking<'a>(function: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    match function.kind() {
        "field_expression" => {
            let name = text(function.child_by_field_name("field")?, source);
            (name == "block_on").then_some(name)
        }
        "scoped_identifier" => {
            let name = last_segment(function, source);
            let threaded = function
                .child_by_field_name("path")
                .is_some_and(|path| last_segment(path, source) == "thread");
            let sleeping = name == "sleep" && threaded;
            (name == "block_on" || sleeping).then_some(name)
        }
        "identifier" => {
            let name = text(function, source);
            let sleeping = name == "sleep" && imports_thread_sleep(function, source);
            (name == "block_on" || sleeping).then_some(name)
        }
        "generic_function" => blocking(function.child_by_field_name("function")?, source),
        _ => None,
    }
}
fn imports_thread_sleep(node: Node<'_>, source: Code<'_>) -> bool {
    let mut root = node;
    while let Some(parent) = root.parent() {
        root = parent;
    }
    let mut pending = vec![(root, false)];
    while let Some((current, imported)) = pending.pop() {
        let inside = imported || current.kind() == "use_declaration";
        let named = current.kind() == "identifier" && text(current, source) == "sleep";
        let found = inside && named && from_thread(current, source);
        if found {
            return true;
        }
        let mut cursor = current.walk();
        pending.extend(
            current
                .named_children(&mut cursor)
                .map(|child| (child, inside)),
        );
    }
    false
}
fn from_thread(name: Node<'_>, source: Code<'_>) -> bool {
    let parent = name.parent();
    let prefix = parent.and_then(|parent| match parent.kind() {
        "scoped_identifier" => parent.child_by_field_name("path"),
        "use_list" => parent
            .parent()
            .and_then(|list| list.child_by_field_name("path")),
        _ => None,
    });
    prefix.is_some_and(|path| path != name && last_segment(path, source) == "thread")
}
