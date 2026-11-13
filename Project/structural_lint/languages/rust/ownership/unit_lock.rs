use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::context::{in_test_context, last_segment};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::ownership::Ownership;
use tree_sitter::Node;

const LOCKS: &[&str] = &["Mutex", "RwLock"];

// DECISION: D155
pub(super) fn declared(generic: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(lock) = generic
        .child_by_field_name("type")
        .map(|path| last_segment(path, source))
        .filter(|name| LOCKS.contains(name))
    else {
        return;
    };
    let unit = generic
        .child_by_field_name("type_arguments")
        .is_some_and(|arguments| sole(arguments).is_some_and(|inner| inner.kind() == "unit_type"));
    if unit {
        report(generic, lock, source, output);
    }
}
pub(super) fn constructed(call: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(function) = call
        .child_by_field_name("function")
        .filter(|function| function.kind() == "scoped_identifier")
    else {
        return;
    };
    let constructor = function
        .child_by_field_name("name")
        .is_some_and(|name| text(name, source) == "new");
    let Some(lock) = function
        .child_by_field_name("path")
        .map(|path| last_segment(path, source))
        .filter(|name| constructor && LOCKS.contains(name))
    else {
        return;
    };
    let unit = call
        .child_by_field_name("arguments")
        .is_some_and(|arguments| {
            sole(arguments).is_some_and(|inner| inner.kind() == "unit_expression")
        });
    let flagged = unit && inferred(call);
    if flagged {
        report(call, lock, source, output);
    }
}
fn sole(list: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = list.walk();
    let items: Vec<_> = list
        .named_children(&mut cursor)
        .filter(|item| !item.is_extra())
        .collect();
    match items.as_slice() {
        [single] => Some(*single),
        _ => None,
    }
}
fn inferred(call: Node<'_>) -> bool {
    let mut current = call;
    while let Some(parent) = current.parent() {
        match parent.kind() {
            "arguments" | "call_expression" => current = parent,
            "let_declaration" => return parent.child_by_field_name("type").is_none(),
            _ => return false,
        }
    }
    false
}
fn report(node: Node<'_>, lock: &str, source: Code<'_>, output: &mut Vec<Measurement>) {
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Ownership(Ownership::UnitLock),
        line: node.start_position().row + 1,
        symbol: lock.into(),
        actual: 1,
    });
}
