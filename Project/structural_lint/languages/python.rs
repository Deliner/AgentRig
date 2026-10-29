use super::{Measurement, Nest, comment, condition, function, nesting, text};
use crate::structural_lint::rules::kind::Kind;
use tree_sitter::Node;

// DECISION: D017
fn method_receiver(node: Node<'_>, source: &str) -> bool {
    let parent = node.parent();
    let decorated = parent.filter(|parent| parent.kind() == "decorated_definition");
    if let Some(wrapper) = decorated {
        let mut cursor = wrapper.walk();
        let static_method = wrapper.named_children(&mut cursor).any(|child| {
            child.kind() == "decorator" && text(child, source).trim() == "@staticmethod"
        });
        if static_method {
            return false;
        }
    }
    let container = decorated.unwrap_or(node).parent();
    container
        .and_then(|block| block.parent())
        .is_some_and(|parent| parent.kind() == "class_definition")
}
// DECISION: D037
fn statement(node: Node<'_>, output: &mut Vec<Measurement>) {
    let mut cursor = node.walk();
    let mut values = node
        .named_children(&mut cursor)
        .filter(|child| !child.is_extra());
    let lone_string =
        values.next().is_some_and(|value| value.kind() == "string") && values.next().is_none();
    if lone_string {
        output.push(Measurement {
            kind: Kind::CommentReference,
            line: node.start_position().row + 1,
            symbol: "docstring".into(),
            actual: 1,
        });
    }
}
pub fn inspect(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    match node.kind() {
        "comment" => comment(node, source, output),
        "expression_statement" => statement(node, output),
        "if_statement" | "elif_clause" => {
            let Some(test) = node.child_by_field_name("condition") else {
                return;
            };
            condition(node, test, output);
        }
        "conditional_expression" => {
            let mut cursor = node.walk();
            let mut operands = node
                .named_children(&mut cursor)
                .filter(|child| !child.is_extra());
            let _consequence = operands.next();
            let Some(test) = operands.next() else {
                return;
            };
            condition(node, test, output);
        }
        "if_clause" => {
            let mut cursor = node.walk();
            let Some(test) = node
                .named_children(&mut cursor)
                .find(|child| !child.is_extra())
            else {
                return;
            };
            condition(node, test, output);
        }
        "function_definition" | "lambda" => {
            let count = parameter_count(node);
            let receiver = method_receiver(node, source) && count > 0;
            function(node, source, count - u64::from(receiver), output);
            nesting(node, source, classify, output);
        }
        _ => {}
    }
}
// DECISION: D061
fn classify(node: Node<'_>) -> Nest {
    match node.kind() {
        "function_definition" | "lambda" => Nest::Boundary,
        "if_statement" | "for_statement" | "while_statement" | "with_statement"
        | "try_statement" | "match_statement" => Nest::Increment,
        _ => Nest::Plain,
    }
}

fn parameter_count(node: Node<'_>) -> u64 {
    node.child_by_field_name("parameters")
        .map(|parameters| {
            let mut cursor = parameters.walk();
            parameters
                .named_children(&mut cursor)
                .filter(|parameter| {
                    !matches!(
                        parameter.kind(),
                        "keyword_separator" | "positional_separator" | "comment"
                    )
                })
                .count() as u64
        })
        .unwrap_or(0)
}
