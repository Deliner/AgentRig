use super::context::{in_test_context, last_segment};
use super::suppression;
use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use tree_sitter::Node;

const UNWRAPPING: [&str; 4] = ["unwrap", "expect", "unwrap_err", "expect_err"];
const PANIC_MACROS: [&str; 4] = ["panic", "todo", "unimplemented", "unreachable"];

// DECISION: D082
// DECISION: D115
pub(super) fn panic_call(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let name = match function.child_by_field_name("field") {
        Some(field) if field.kind() == "field_identifier" => text(field, source),
        Some(_) => return,
        None => last_segment(function, source),
    };
    let method = function.kind() == "field_expression";
    let path = matches!(function.kind(), "identifier" | "scoped_identifier");
    let unwrapping = method && UNWRAPPING.contains(&name);
    let resuming = path && name == "resume_unwind";
    let panic = unwrapping || resuming;
    if panic {
        report_panic(node, source, name, output);
    }
}
pub(super) fn panic_macro(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(path) = node.child_by_field_name("macro") else {
        return;
    };
    let name = text(path, source);
    let panic = PANIC_MACROS.contains(&name);
    if panic {
        report_panic(node, source, name, output);
    }
}
// DECISION: D148
pub(super) fn definition_tokens(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let defined = std::iter::successors(node.parent(), Node::parent)
        .any(|ancestor| ancestor.kind() == "macro_definition");
    let outside = !defined;
    if outside {
        return;
    }
    let mut cursor = node.walk();
    let tokens: Vec<_> = node.children(&mut cursor).collect();
    for (token, symbol) in tokens
        .windows(3)
        .filter_map(|window| panic_token(window, source))
    {
        report_panic(token, source, symbol, output);
    }
}
fn panic_token<'a>(window: &[Node<'a>], source: Code<'a>) -> Option<(Node<'a>, &'a str)> {
    let [first, second, third] = *window else {
        return None;
    };
    let (head, name) = (text(first, source), text(second, source));
    let tree = third.kind() == "token_tree";
    let call = tree && third.child(0).is_some_and(|open| open.kind() == "(");
    let method = first.kind() == ".";
    let unwrapping = method && call && UNWRAPPING.contains(&name);
    let unwinding = !method && call && name == "resume_unwind";
    let invoked = second.kind() == "!" && tree && PANIC_MACROS.contains(&head);
    if invoked {
        return Some((first, head));
    }
    (unwrapping || unwinding).then_some((second, name))
}
fn report_panic(node: Node<'_>, source: Code<'_>, symbol: &str, output: &mut Vec<Measurement>) {
    let allowed = justified(node, source)
        || in_test_context(node, source)
        || suppression::clippy_allowed(node, source, symbol);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::PanicPath),
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
// DECISION: D115
pub(super) fn catch(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let Some(symbol) = catch_site(function, source) else {
        return;
    };
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::UnwindJustification),
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn catch_site<'a>(function: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    match function.kind() {
        "field_expression" => {
            let name = text(function.child_by_field_name("field")?, source);
            (name == "catch_unwind").then_some(name)
        }
        "identifier" | "scoped_identifier" => {
            let name = last_segment(function, source);
            matches!(name, "catch_unwind" | "AssertUnwindSafe").then_some(name)
        }
        _ => None,
    }
}
