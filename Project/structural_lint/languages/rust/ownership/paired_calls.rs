use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::context::in_test_context;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::ownership::{Ownership, Pair};
use tree_sitter::Node;

// DECISION: D156
pub(super) fn inspect(
    block: Node<'_>,
    source: Code<'_>,
    pairs: &[Pair],
    output: &mut Vec<Measurement>,
) {
    let mut cursor = block.walk();
    let statements: Vec<_> = block
        .named_children(&mut cursor)
        .filter(|statement| !statement.is_extra())
        .collect();
    for (index, statement) in statements.iter().enumerate() {
        let rest = statements.get(index + 1..).unwrap_or_default();
        let unguarded: Vec<_> = pairs
            .iter()
            .filter(|pair| escapes(*statement, rest, pair, source))
            .collect();
        let allowed = unguarded.is_empty()
            || in_test_context(*statement, source)
            || justified(*statement, source);
        if allowed {
            continue;
        }
        output.extend(unguarded.into_iter().map(|pair| Measurement {
            kind: Kind::Ownership(Ownership::PairedCalls),
            line: statement.start_position().row + 1,
            symbol: pair.open.clone(),
            actual: 1,
        }));
    }
}
fn escapes(statement: Node<'_>, rest: &[Node<'_>], pair: &Pair, source: Code<'_>) -> bool {
    let Some(receiver) = called(statement, &pair.open, source) else {
        return false;
    };
    let closing = rest
        .iter()
        .position(|later| called(*later, &pair.close, source) == Some(receiver));
    let between = closing
        .and_then(|closing| rest.get(..closing))
        .unwrap_or_default();
    between.iter().any(|node| exits(*node))
}
fn called<'a>(node: Node<'_>, method: &str, source: Code<'a>) -> Option<&'a str> {
    let closure = node.kind() == "closure_expression";
    if closure {
        return None;
    }
    let field = node.child_by_field_name("function").filter(|function| {
        node.kind() == "call_expression" && function.kind() == "field_expression"
    });
    let receiver = field
        .filter(|function| {
            function
                .child_by_field_name("field")
                .is_some_and(|name| text(name, source) == method)
        })
        .and_then(|function| function.child_by_field_name("value"));
    if let Some(receiver) = receiver {
        return Some(text(receiver, source));
    }
    let mut cursor = node.walk();
    let children: Vec<_> = node.named_children(&mut cursor).collect();
    children
        .into_iter()
        .find_map(|child| called(child, method, source))
}
fn exits(node: Node<'_>) -> bool {
    match node.kind() {
        "closure_expression" | "function_item" => false,
        "try_expression" | "return_expression" | "break_expression" | "continue_expression" => true,
        _ => {
            let mut cursor = node.walk();
            let children: Vec<_> = node.named_children(&mut cursor).collect();
            children.into_iter().any(exits)
        }
    }
}
