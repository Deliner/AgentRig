use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::context::in_test_context;
use crate::structural_lint::rules::control_flow::ControlFlow;
use crate::structural_lint::rules::kind::Kind;
use tree_sitter::Node;

const EXPRESSIONS: [&str; 2] = ["expr", "expr_2021"];

pub(super) fn inspect(rule: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let (Some(matcher), Some(transcriber)) = (
        rule.child_by_field_name("left"),
        rule.child_by_field_name("right"),
    ) else {
        return;
    };
    let mut bound = Vec::new();
    expressions(matcher, source, &mut bound);
    for name in bound {
        let mut found = Vec::new();
        uses(transcriber, name, source, &mut found);
        let Some(&second) = found.get(1) else {
            continue;
        };
        let allowed = justified(second, source) || in_test_context(second, source);
        if allowed {
            continue;
        }
        output.push(Measurement {
            kind: Kind::ControlFlow(ControlFlow::RepeatedFragment),
            line: second.start_position().row + 1,
            symbol: name.into(),
            actual: 1,
        });
    }
}
fn expressions<'a>(node: Node<'_>, source: Code<'a>, bound: &mut Vec<&'a str>) {
    let binding = node.kind() == "token_binding_pattern";
    let name = node.child_by_field_name("name");
    let fragment = node.child_by_field_name("type");
    if let (true, Some(name), Some(fragment)) = (binding, name, fragment) {
        let expression = EXPRESSIONS.contains(&text(fragment, source));
        if expression {
            bound.push(text(name, source));
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        expressions(child, source, bound);
    }
}
fn uses<'a>(node: Node<'a>, name: &str, source: Code<'_>, found: &mut Vec<Node<'a>>) {
    let named = node.kind() == "metavariable" && text(node, source) == name;
    if named {
        found.push(node);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        let quoted = stringified(child, source);
        if quoted {
            continue;
        }
        uses(child, name, source, found);
    }
}
fn stringified(tree: Node<'_>, source: Code<'_>) -> bool {
    let bang = tree.prev_sibling().filter(|token| token.kind() == "!");
    let head = bang.and_then(|token| token.prev_sibling());
    tree.kind() == "token_tree" && head.is_some_and(|token| text(token, source) == "stringify")
}
