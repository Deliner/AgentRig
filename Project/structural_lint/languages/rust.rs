use super::{
    Branch, Measurement, Nest, boolean_parameters, branching, comment, condition, credential,
    function, justified, nesting, text,
};
use crate::structural_lint::rules::kind::Kind;
use tree_sitter::Node;

// DECISION: D017
// DECISION: D037
pub fn inspect(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    match node.kind() {
        "line_comment" | "block_comment" => comment(node, source, output),
        "string_literal" | "raw_string_literal" => credential(node, source, output),
        "unsafe_block" => unsafe_justification(node, source, output),
        "attribute_item" => disabled_test(node, source, output),
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
            declaration(node, source, output)
        }
        _ => {}
    }
}
fn declaration(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
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
    branching(node, source, branch, output);
    let unsafe_declaration = unsafe_function(node, source);
    if unsafe_declaration {
        unsafe_justification(node, source, output);
    }
}
// DECISION: D065
fn unsafe_function(node: Node<'_>, source: &str) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor).any(|child| {
        child.kind() == "function_modifiers"
            && text(child, source)
                .split_whitespace()
                .any(|word| word == "unsafe")
    })
}
fn disabled_test(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let ignored = node
        .named_children(&mut node.walk())
        .find(|child| child.kind() == "attribute")
        .and_then(|attribute| attribute.named_child(0))
        .map(|name| text(name, source))
        .is_some_and(|name| name == "ignore");
    let unjustified = ignored && !justified(node, source);
    if unjustified {
        output.push(Measurement {
            kind: Kind::DisabledTest,
            line: node.start_position().row + 1,
            symbol: "ignore".into(),
            actual: 1,
        });
    }
}
fn unsafe_justification(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let unjustified = !justified(node, source);
    if unjustified {
        output.push(Measurement {
            kind: Kind::UnsafeJustification,
            line: node.start_position().row + 1,
            symbol: "unsafe".into(),
            actual: 1,
        });
    }
}
// DECISION: D063
fn branch(node: Node<'_>) -> Branch {
    match node.kind() {
        "function_item" | "function_signature_item" | "closure_expression" => Branch::Boundary,
        "if_expression" | "match_expression" => Branch::Decision,
        _ => Branch::Plain,
    }
}
// DECISION: D062
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
