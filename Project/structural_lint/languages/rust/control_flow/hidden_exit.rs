use crate::structural_lint::languages::measure::{Code, Measurement, justified, reparse, text};
use crate::structural_lint::languages::rust::context::in_test_context;
use crate::structural_lint::rule_identities::control_flow::ControlFlow;
use crate::structural_lint::rule_identities::kind::Kind;
use std::path::Path;
use tree_sitter::Node;

type Exit = (&'static str, &'static str);

const EXIT_WORDS: [&str; 11] = [
    "bail",
    "ensure",
    "require",
    "return",
    "try",
    "break",
    "continue",
    "exit",
    "throw",
    "fail",
    "propagate",
];
const BOUNDARIES: [&str; 3] = ["closure_expression", "function_item", "async_block"];
const LOOPS: [&str; 3] = ["loop_expression", "while_expression", "for_expression"];
const FUNCTION_EXITS: [Exit; 2] = [("return_expression", "return"), ("try_expression", "?")];
const LOOP_EXITS: [Exit; 2] = [
    ("break_expression", "break"),
    ("continue_expression", "continue"),
];

// DECISION: D171
pub(super) fn inspect(rule: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(transcriber) = rule.child_by_field_name("right") else {
        return;
    };
    let skipped = in_test_context(rule, source) || exit_named(rule, source);
    if skipped {
        return;
    }
    let Some(body) = blanked(transcriber, source) else {
        return;
    };
    let fragment = ["fn probe()", &body].concat();
    let Some(parsed) = reparse(Path::new("macro.rs"), &fragment) else {
        return;
    };
    let Some(block) = parsed
        .root_node()
        .named_child(0)
        .and_then(|probe| probe.child_by_field_name("body"))
    else {
        return;
    };
    let code = Code::new(&fragment);
    let mut exits = Vec::new();
    collect(block, &LOOP_EXITS, &mut exits);
    for (exit, symbol) in exits {
        let allowed = justified(exit, code);
        if allowed {
            continue;
        }
        output.push(Measurement {
            kind: Kind::ControlFlow(ControlFlow::HiddenExit),
            line: transcriber.start_position().row + exit.start_position().row + 1,
            symbol: symbol.into(),
            actual: 1,
        });
    }
}
fn exit_named(rule: Node<'_>, source: Code<'_>) -> bool {
    rule.parent()
        .and_then(|definition| definition.child_by_field_name("name"))
        .is_some_and(|name| {
            text(name, source)
                .split('_')
                .any(|part| EXIT_WORDS.contains(&part))
        })
}
fn collect<'a>(node: Node<'a>, leaving: &[Exit], exits: &mut Vec<(Node<'a>, &'static str)>) {
    let kind = node.kind();
    let exit = FUNCTION_EXITS
        .iter()
        .chain(leaving)
        .find(|(exit, _)| *exit == kind);
    if let Some(&(_, symbol)) = exit {
        exits.push((node, symbol));
    }
    let looping = LOOPS.contains(&kind);
    let inner = if looping { &[] } else { leaving };
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        let bounded = BOUNDARIES.contains(&child.kind());
        if bounded {
            continue;
        }
        collect(child, inner, exits);
    }
}
fn blanked(transcriber: Node<'_>, source: Code<'_>) -> Option<String> {
    let start = transcriber.start_byte();
    let text = source.as_str().get(start..transcriber.end_byte())?;
    let mut bytes = text.as_bytes().to_vec();
    let mut spans = Vec::new();
    markers(transcriber, &mut spans);
    for (from, to) in spans {
        let span = bytes.get_mut(from.checked_sub(start)?..to.checked_sub(start)?)?;
        for byte in span.iter_mut().filter(|byte| **byte != b'\n') {
            *byte = b' ';
        }
    }
    *bytes.first_mut()? = b'{';
    *bytes.last_mut()? = b'}';
    String::from_utf8(bytes).ok()
}
fn markers(node: Node<'_>, spans: &mut Vec<(usize, usize)>) {
    let variable = node.kind() == "metavariable";
    if variable {
        spans.push((node.start_byte(), node.start_byte() + 1));
    }
    let repeated = node.kind() == "token_repetition";
    if repeated {
        repetition(node, spans);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        markers(child, spans);
    }
}
fn repetition(node: Node<'_>, spans: &mut Vec<(usize, usize)>) {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    let close = children.iter().position(|child| child.kind() == ")");
    for (index, child) in children.iter().enumerate() {
        let opening = index < 2;
        let closing = close.is_some_and(|close| index >= close);
        let blank = opening || closing;
        if blank {
            spans.push((child.start_byte(), child.end_byte()));
        }
    }
}
