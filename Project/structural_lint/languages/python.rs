use super::measure::{
    Analysis, Branch, Measurement, Nest, boolean_expression, boolean_parameters, branching,
    comment, condition, credential, function, justified, mixed_junction, nesting, text,
};
use crate::structural_lint::languages::measure::Code;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::ownership::Pair;
use crate::structural_lint::rule_identities::reference::Reference;
use tree_sitter::Node;

// DECISION: D017
fn method_receiver(node: Node<'_>, source: Code<'_>) -> bool {
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
            kind: Kind::Reference(Reference::Comment),
            line: node.start_position().row + 1,
            symbol: "docstring".into(),
            actual: 1,
        });
    }
}
pub fn inspect(node: Node<'_>, source: Code<'_>, _pairs: &[Pair], analysis: &mut Analysis) {
    analysis.mixed_junctions.extend(mixed_junction(node));
    let output = &mut analysis.measurements;
    match node.kind() {
        "comment" => comment(node, source, output),
        "string" => credential(node, source, output),
        "expression_statement" => statement(node, output),
        "except_clause" => silent_except(node, source, output),
        "decorator" => disabled_test(node, source, output),
        "import_from_statement" => wildcard_import(node, source, output),
        "boolean_operator" => boolean_expression(node, output),
        "if_statement" | "elif_clause" => {
            let Some(test) = node.child_by_field_name("condition") else {
                return;
            };
            condition(test, output);
        }
        "conditional_expression" => ternary(node, output),
        "if_clause" => {
            let mut cursor = node.walk();
            let Some(test) = node
                .named_children(&mut cursor)
                .find(|child| !child.is_extra())
            else {
                return;
            };
            condition(test, output);
        }
        "function_definition" | "lambda" => measure(node, source, output),
        _ => {}
    }
}
fn ternary(node: Node<'_>, output: &mut Vec<Measurement>) {
    let mut cursor = node.walk();
    let mut operands = node
        .named_children(&mut cursor)
        .filter(|child| !child.is_extra());
    let _consequence = operands.next();
    let Some(test) = operands.next() else {
        return;
    };
    condition(test, output);
}
// DECISION: D069
// DECISION: D074
fn disabled_test(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(expression) = node.named_child(0) else {
        return;
    };
    let call = expression.kind() == "call";
    let callee = if call {
        expression.child_by_field_name("function")
    } else {
        Some(expression)
    };
    let disabled = callee
        .map(|callee| text(callee, source))
        .is_some_and(|path| matches!(path, "pytest.mark.skip" | "pytest.mark.skipif"));
    let unjustified = disabled && !justified(node, source);
    if unjustified {
        output.push(Measurement {
            kind: Kind::Reference(Reference::DisabledTest),
            line: node.start_position().row + 1,
            symbol: "skip".into(),
            actual: 1,
        });
    }
}
// DECISION: D085
fn wildcard_import(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let mut cursor = node.walk();
    let glob = node
        .named_children(&mut cursor)
        .any(|child| child.kind() == "wildcard_import");
    let unjustified = glob && !justified(node, source);
    if unjustified {
        output.push(Measurement {
            kind: Kind::Reference(Reference::WildcardImport),
            line: node.start_position().row + 1,
            symbol: "glob".into(),
            actual: 1,
        });
    }
}
fn silent_except(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let mut cursor = node.walk();
    let Some(body) = node
        .named_children(&mut cursor)
        .find(|child| child.kind() == "block")
    else {
        return;
    };
    let mut inner = body.walk();
    let mut statements = body
        .named_children(&mut inner)
        .filter(|child| !child.is_extra());
    let Some(first) = statements.next() else {
        return;
    };
    let silent = first.kind() == "pass_statement" && statements.next().is_none();
    let swallowed = silent && !justified(first, source);
    if swallowed {
        output.push(Measurement {
            kind: Kind::Reference(Reference::SilentError),
            line: node.start_position().row + 1,
            symbol: "except".into(),
            actual: 1,
        });
    }
}
fn measure(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let count = parameter_count(node);
    let receiver = method_receiver(node, source) && count > 0;
    function(node, source, count - usize::from(receiver), output);
    nesting(node, source, classify, output);
    boolean_parameters(node, source, flag, output);
    branching(node, source, branch, output);
}
// DECISION: D063
fn branch(node: Node<'_>) -> Branch {
    match node.kind() {
        "function_definition" | "lambda" => Branch::Boundary,
        "if_statement" | "elif_clause" | "conditional_expression" | "match_statement" => {
            Branch::Decision
        }
        _ => Branch::Plain,
    }
}
// DECISION: D062
fn flag(parameter: Node<'_>, source: Code<'_>) -> bool {
    let annotated = parameter
        .child_by_field_name("type")
        .is_some_and(|annotation| text(annotation, source).trim() == "bool");
    let defaulted = parameter
        .child_by_field_name("value")
        .is_some_and(|value| matches!(value.kind(), "true" | "false"));
    annotated || defaulted
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

fn parameter_count(node: Node<'_>) -> usize {
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
                .count()
        })
        .unwrap_or(0)
}
