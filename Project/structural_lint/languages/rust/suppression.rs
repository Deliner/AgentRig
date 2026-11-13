use super::context::attributed;
use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use tree_sitter::Node;

// DECISION: D074
// DECISION: D080
// DECISION: D098
const SUPPRESSIONS: &[&str] = &["allow", "expect", "ignore"];

pub(super) fn attribute(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(name) = suppression(node, source) else {
        return;
    };
    let kind = match name {
        "ignore" => Kind::Reference(Reference::DisabledTest),
        _ => Kind::Reference(Reference::AllowJustification),
    };
    let unjustified = !justified(node, source);
    if unjustified {
        output.push(Measurement {
            kind,
            line: node.start_position().row + 1,
            symbol: name.into(),
            actual: 1,
        });
    }
}
// DECISION: D147
pub(super) fn clippy_allowed(node: Node<'_>, source: Code<'_>, symbol: &str) -> bool {
    let Some(lint) = clippy_lint(symbol) else {
        return false;
    };
    std::iter::successors(node.parent(), Node::parent).any(|item| {
        attributed(item, source, |attribute, source| {
            allows(attribute, source, lint)
        })
    })
}
fn clippy_lint(symbol: &str) -> Option<&'static str> {
    match symbol {
        "unwrap" | "unwrap_err" => Some("clippy::unwrap_used"),
        "expect" | "expect_err" => Some("clippy::expect_used"),
        "panic" => Some("clippy::panic"),
        "todo" => Some("clippy::todo"),
        "unimplemented" => Some("clippy::unimplemented"),
        "unreachable" => Some("clippy::unreachable"),
        _ => None,
    }
}
fn allows(item: Node<'_>, source: Code<'_>, lint: &str) -> bool {
    let Some(attribute) = item
        .named_children(&mut item.walk())
        .find(|child| child.kind() == "attribute")
    else {
        return false;
    };
    let suppressing = attribute
        .named_child(0)
        .is_some_and(|name| matches!(text(name, source), "allow" | "expect"));
    let lints = attribute
        .child_by_field_name("arguments")
        .map_or("", |arguments| text(arguments, source));
    let listed = lints
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .any(|name| name.trim() == lint);
    suppressing && listed && justified(item, source)
}
fn suppression<'a>(node: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let attribute = node
        .named_children(&mut node.walk())
        .find(|child| child.kind() == "attribute")?;
    let name = text(attribute.named_child(0)?, source);
    let conditional = name == "cfg_attr";
    if conditional {
        return conditional_head(attribute.child_by_field_name("arguments")?, source);
    }
    SUPPRESSIONS.contains(&name).then_some(name)
}
fn conditional_head<'a>(arguments: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let mut cursor = arguments.walk();
    let mut argument = false;
    for token in arguments.children(&mut cursor) {
        let separator = token.kind() == ",";
        if separator {
            argument = true;
            continue;
        }
        let name = text(token, source);
        let head = argument && token.kind() == "identifier" && SUPPRESSIONS.contains(&name);
        if head {
            return Some(name);
        }
        argument = false;
    }
    None
}
