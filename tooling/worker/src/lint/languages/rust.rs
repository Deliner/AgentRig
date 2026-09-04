use super::{Measurement, condition, function};
use tree_sitter::Node;

// DECISION: D017
pub fn inspect(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    match node.kind() {
        "if_expression" => {
            let test = node
                .child_by_field_name("condition")
                .expect("parsed if condition");
            let binding = test.kind() == "let_condition";
            if binding {
                return;
            }
            condition(node, test, output);
        }
        "function_item" | "function_signature_item" | "closure_expression" => {
            let parameters = node
                .child_by_field_name("parameters")
                .expect("parsed parameters");
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
        }
        _ => {}
    }
}
