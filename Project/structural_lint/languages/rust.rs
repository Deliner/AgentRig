use super::{Measurement, Nest, boolean_parameters, comment, condition, function, nesting, text};
use tree_sitter::Node;

// DECISION: D017
// DECISION: D037
pub fn inspect(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    match node.kind() {
        "line_comment" | "block_comment" => comment(node, source, output),
        "if_expression" => {
            let Some(test) = node.child_by_field_name("condition") else {
                return;
            };
            let binding = test.kind() == "let_condition";
            if binding {
                return;
            }
            condition(node, test, output);
        }
        "function_item" | "function_signature_item" | "closure_expression" => {
            let Some(parameters) = node.child_by_field_name("parameters") else {
                return;
            };
            let mut cursor = parameters.walk();
            let count = parameters
                .named_children(&mut cursor)
                .filter(|parameter| {
                    let excluded = matches!(
                        parameter.kind(),
                        "self_parameter" | "attribute_item" | "line_comment" | "block_comment"
                    ) || parameter
                        .child_by_field_name("pattern")
                        .is_some_and(|pattern| pattern.kind() == "self");
                    !excluded
                })
                .count() as u64;
            function(node, source, count, output);
            nesting(node, source, classify, output);
            boolean_parameters(node, source, flag, output);
        }
        _ => {}
    }
}
fn flag(parameter: Node<'_>, source: &str) -> bool {
    parameter
        .child_by_field_name("type")
        .is_some_and(|annotation| {
            annotation.kind() == "primitive_type" && text(annotation, source) == "bool"
        })
}
// DECISION: D061
fn classify(node: Node<'_>) -> Nest {
    match node.kind() {
        "function_item" | "function_signature_item" | "closure_expression" => Nest::Boundary,
        "if_expression" => {
            let continuation = node
                .parent()
                .is_some_and(|parent| parent.kind() == "else_clause");
            if continuation {
                Nest::Plain
            } else {
                Nest::Increment
            }
        }
        "match_expression" | "while_expression" | "loop_expression" | "for_expression" => {
            Nest::Increment
        }
        _ => Nest::Plain,
    }
}
