use super::exemptions::{in_test_context, last_segment};
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, text};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
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
        "call_expression" => parallel_index(node, source, output),
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
    let mut cursor = invocation.walk();
    let Some(tree) = invocation
        .named_children(&mut cursor)
        .find(|child| child.kind() == "token_tree")
    else {
        return;
    };
    let mut tokens = tree.walk();
    let template = tree
        .named_children(&mut tokens)
        .find(|token| matches!(token.kind(), "string_literal" | "raw_string_literal"));
    let positional = template.map_or(0, |template| positional(text(template, source)));
    let allowed = positional < 2 || in_test_context(invocation, source);
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
    let allowed = in_test_context(literal, source);
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
// DECISION: D211
fn parallel_index(call: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(searched) = searched(call, source) else {
        return;
    };
    let mut indices = Vec::new();
    let mut top = call;
    while let Some(next) = top.parent().and_then(chained) {
        let mut cursor = next.walk();
        let closures: Vec<Node<'_>> = next
            .child_by_field_name("arguments")
            .map(|arguments| arguments.named_children(&mut cursor).collect())
            .unwrap_or_default();
        for closure in closures {
            let body: Vec<Node<'_>> = closure.child_by_field_name("body").into_iter().collect();
            let parameters = closure.child_by_field_name("parameters");
            parameters.inspect(|parameters| track(*parameters, &body, source, &mut indices));
        }
        top = next;
    }
    indices.push(top);
    bound(top, source, &mut indices);
    let reads = indices
        .into_iter()
        .filter_map(|index| indexed(index, source))
        .filter(|collection| {
            let allowed =
                text(*collection, source) == searched || in_test_context(*collection, source);
            !allowed
        });
    output.extend(reads.map(|collection| Measurement {
        kind: Kind::Reference(Reference::ParallelIndex),
        line: collection.start_position().row + 1,
        symbol: "position".into(),
        actual: 1,
    }));
}
fn searched<'a>(call: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let iterator = receiver(call, source, &["position"], 1)?;
    let collection = receiver(iterator, source, &["iter", "iter_mut", "into_iter"], 0)?;
    Some(text(collection, source))
}
fn receiver<'t>(
    call: Node<'t>,
    source: Code<'_>,
    methods: &[&str],
    arity: usize,
) -> Option<Node<'t>> {
    let function = call.child_by_field_name("function").filter(|function| {
        call.kind() == "call_expression" && function.kind() == "field_expression"
    })?;
    let named = function
        .child_by_field_name("field")
        .is_some_and(|field| methods.contains(&text(field, source)));
    let counted = call
        .child_by_field_name("arguments")
        .is_some_and(|arguments| arguments.named_child_count() == arity);
    let matching = named && counted;
    matching
        .then(|| function.child_by_field_name("value"))
        .flatten()
}
fn chained(parent: Node<'_>) -> Option<Node<'_>> {
    match parent.kind() {
        "try_expression" => Some(parent),
        "field_expression" => parent
            .parent()
            .filter(|call| call.kind() == "call_expression"),
        _ => None,
    }
}
fn bound<'t>(top: Node<'t>, source: Code<'_>, indices: &mut Vec<Node<'t>>) {
    let Some(binding) = top.parent() else {
        return;
    };
    let scope: Vec<Node<'t>> = match binding.kind() {
        "let_declaration" => {
            std::iter::successors(binding.next_named_sibling(), Node::next_named_sibling).collect()
        }
        "let_condition" => std::iter::successors(binding.parent(), Node::parent)
            .find(|owner| owner.kind() != "let_chain")
            .and_then(|owner| owner.child_by_field_name("consequence"))
            .into_iter()
            .collect(),
        _ => Vec::new(),
    };
    let pattern = binding.child_by_field_name("pattern");
    pattern.inspect(|pattern| track(*pattern, &scope, source, indices));
}
fn track<'t>(pattern: Node<'_>, scope: &[Node<'t>], source: Code<'_>, indices: &mut Vec<Node<'t>>) {
    let mut names = BTreeSet::new();
    named(pattern, source, &mut names);
    for node in scope {
        used(*node, &names, source, indices);
    }
}
fn named<'a>(pattern: Node<'_>, source: Code<'a>, names: &mut BTreeSet<&'a str>) {
    let typed = pattern
        .parent()
        .and_then(|parent| parent.child_by_field_name("type"))
        .is_some_and(|annotation| annotation == pattern);
    let binding = pattern.kind() == "identifier" && !typed;
    if binding {
        names.insert(text(pattern, source));
    }
    let mut cursor = pattern.walk();
    for child in pattern.named_children(&mut cursor) {
        named(child, source, names);
    }
}
fn used<'t>(node: Node<'t>, names: &BTreeSet<&str>, source: Code<'_>, indices: &mut Vec<Node<'t>>) {
    let named = node.kind() == "identifier" && names.contains(text(node, source));
    if named {
        indices.push(node);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        used(child, names, source, indices);
    }
}
fn indexed<'t>(index: Node<'t>, source: Code<'_>) -> Option<Node<'t>> {
    let parent = index.parent()?;
    match parent.kind() {
        "index_expression" => {
            let position = parent.named_child(1).is_some_and(|child| child == index);
            position.then(|| parent.named_child(0)).flatten()
        }
        "arguments" => receiver(parent.parent()?, source, &["get", "get_mut"], 1),
        _ => None,
    }
}
