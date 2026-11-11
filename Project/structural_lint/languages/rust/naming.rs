use super::context::{in_test_context, last_segment};
use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use std::collections::BTreeSet;
use tree_sitter::Node;

const FORMATTERS: &[&str] = &[
    "format",
    "print",
    "println",
    "eprint",
    "eprintln",
    "write",
    "writeln",
    "panic",
    "format_args",
];

pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    match node.kind() {
        "macro_invocation" => positional_format(node, source, output),
        "struct_expression" => crossed_field(node, source, output),
        _ => {}
    }
}
// DECISION: D156
fn positional_format(invocation: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(name) = invocation
        .child_by_field_name("macro")
        .map(|path| last_segment(path, source))
        .filter(|name| FORMATTERS.contains(name))
    else {
        return;
    };
    let positional = template(invocation).map_or(0, |template| positional(text(template, source)));
    let allowed =
        positional < 2 || in_test_context(invocation, source) || justified(invocation, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::PositionalFormat),
        line: invocation.start_position().row + 1,
        symbol: name.into(),
        actual: 1,
    });
}
pub(super) fn template(invocation: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = invocation.walk();
    let tree = invocation
        .named_children(&mut cursor)
        .find(|child| child.kind() == "token_tree")?;
    let mut tokens = tree.walk();
    tree.named_children(&mut tokens)
        .find(|token| matches!(token.kind(), "string_literal" | "raw_string_literal"))
}
fn positional(template: &str) -> usize {
    let mut count = 0;
    let mut rest = template;
    while let Some((_, after)) = rest.split_once('{') {
        if let Some(escaped) = after.strip_prefix('{') {
            rest = escaped;
            continue;
        }
        let Some((placeholder, remaining)) = after.split_once('}') else {
            break;
        };
        let argument = placeholder.split(':').next().unwrap_or_default();
        let unnamed = argument.chars().all(|letter| letter.is_ascii_digit());
        count += usize::from(unnamed);
        rest = remaining;
    }
    count
}
// DECISION: D156
fn crossed_field(literal: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(body) = literal.child_by_field_name("body") else {
        return;
    };
    let mut cursor = body.walk();
    let initializers: Vec<_> = body.named_children(&mut cursor).collect();
    let fields: BTreeSet<&str> = initializers
        .iter()
        .filter_map(|initializer| field(*initializer, source))
        .collect();
    let crossed = initializers.iter().find(|initializer| {
        let value = initializer
            .child_by_field_name("value")
            .filter(|value| value.kind() == "identifier")
            .map(|value| text(value, source));
        let own = field(**initializer, source);
        value.is_some_and(|value| Some(value) != own && fields.contains(value))
    });
    let Some(crossed) = crossed else {
        return;
    };
    let allowed = in_test_context(literal, source) || justified(literal, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::CrossedField),
        line: crossed.start_position().row + 1,
        symbol: text(*crossed, source).split_whitespace().collect(),
        actual: 1,
    });
}
fn field<'a>(initializer: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let name = match initializer.kind() {
        "field_initializer" => initializer.child_by_field_name("field"),
        "shorthand_field_initializer" => {
            let mut cursor = initializer.walk();
            initializer
                .named_children(&mut cursor)
                .find(|child| child.kind() == "identifier")
        }
        _ => None,
    };
    name.map(|name| text(name, source))
}
