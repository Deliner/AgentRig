mod blocking_async;
mod control_flow;
mod errors;
mod exemptions;
mod naming;
mod ownership;
mod panic_surface;
mod shell_command;
mod signature;
mod std_forms;
mod super_reach;
mod suppression;
mod test_code;
mod unsafe_site;
use crate::structural_lint::languages::neutral_metrics::{
    Analysis, Branch, Code, Context, Measurement, Nest, Project, boolean_expression,
    boolean_parameters, branching, comment, condition, credential, function_lines, mixed_junction,
    nesting, parameter_count, reparse, text,
};
use crate::structural_lint::rule_identities::{Kind, Reference, Structure};
pub(crate) use exemptions::{ENTRY_FILES, in_test_context};
use exemptions::{foreign_signature, in_debug_only_context};
use tree_sitter::Node;
// DECISION: D017
// DECISION: D037
pub fn inspect(node: Node<'_>, source: Code<'_>, context: Context<'_>, analysis: &mut Analysis) {
    analysis.mixed_junctions.extend(mixed_junction(node));
    let output = &mut analysis.measurements;
    signature::inspect(node, source, context.project, output);
    ownership::inspect(node, source, context.pairs, output);
    control_flow::inspect(node, source, output);
    errors::inspect(node, source, output);
    naming::inspect(node, source, output);
    test_code::inspect(node, source, output);
    std_forms::inspect(node, source, output);
    match node.kind() {
        "line_comment" | "block_comment" => comment(node, source, output),
        "string_literal" | "raw_string_literal" => literal(node, source, output),
        "unsafe_block" | "impl_item" | "trait_item" | "token_tree" | "token_repetition" => {
            unsafe_site::inspect(node, source, output);
            panic_surface::definition_tokens(node, source, output);
        }
        "attribute_item" | "inner_attribute_item" => suppression::attribute(node, source, output),
        "static_item" => unsafe_site::inspect(node, source, output),
        "macro_invocation" => {
            debug_macro(node, source, output);
            assertion_macro(node, source, output);
            panic_surface::panic_macro(node, source, output);
            macro_body(node, source, output);
        }
        "call_expression" => call(node, source, output),
        "scoped_identifier" => super_reach::inspect(node, source, output),
        "use_wildcard" => wildcard_use(node, source, output),
        "if_expression" => conditional(node, output),
        "binary_expression" | "let_chain" => boolean_expression(node, output),
        "function_item" | "function_signature_item" | "closure_expression" => {
            declaration(node, source, context.project, output);
            unsafe_site::inspect(node, source, output);
        }
        _ => {}
    }
}
// DECISION: D122
fn literal(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    credential(node, source, output);
    let exempt = in_test_context(node, source);
    if exempt {
        return;
    }
    let first = node.start_position().row;
    output.push(Measurement {
        kind: Kind::Structure(Structure::EmbeddedText),
        line: first + 1,
        symbol: "literal".into(),
        actual: node.end_position().row - first + 1,
    });
}
fn call(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    panic_surface::panic_call(node, source, output);
    std_forms::call(node, source, output);
    panic_surface::catch(node, source, output);
    blocking_async::inspect(node, source, output);
    shell_command::inspect(node, source, output);
}
fn conditional(node: Node<'_>, output: &mut Vec<Measurement>) {
    let Some(test) = node.child_by_field_name("condition") else {
        return;
    };
    let binding = test.kind() == "let_condition";
    if binding {
        return;
    }
    condition(test, output);
}
fn declaration(
    node: Node<'_>,
    source: Code<'_>,
    project: &dyn Project,
    output: &mut Vec<Measurement>,
) {
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
        .count();
    // DECISION: D229
    let judged = !foreign_signature(node, source, project);
    if judged {
        parameter_count(node, source, count, output);
        boolean_parameters(node, source, flag, output);
    }
    function_lines(node, source, output);
    nesting(node, source, classify, output);
    branching(node, source, branch, output);
}
// DECISION: D081
fn debug_macro(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(path) = node.child_by_field_name("macro") else {
        return;
    };
    let debug = text(path, source) == "dbg";
    if debug {
        output.push(Measurement {
            kind: Kind::Reference(Reference::DebugArtifact),
            line: node.start_position().row + 1,
            symbol: "dbg".into(),
            actual: 1,
        });
    }
}
// DECISION: D084
// DECISION: D115
fn assertion_macro(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(path) = node.child_by_field_name("macro") else {
        return;
    };
    let name = text(path, source);
    let debug = matches!(name, "debug_assert" | "debug_assert_eq" | "debug_assert_ne");
    let gated =
        matches!(name, "assert" | "assert_eq" | "assert_ne") && in_debug_only_context(node, source);
    let assertion = debug || gated;
    if assertion {
        report_assertion(node, source, name, output);
    }
}
fn report_assertion(node: Node<'_>, source: Code<'_>, symbol: &str, output: &mut Vec<Measurement>) {
    let allowed = in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::DebugAssertion),
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
// DECISION: D094
// DECISION: D147
fn macro_body(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let exempt = in_test_context(node, source);
    if exempt {
        return;
    }
    let scoped = fragment_measurements(node, source);
    output.extend(scoped.into_iter().filter(|measurement| {
        let panic = measurement.kind == Kind::Reference(Reference::PanicPath);
        !(panic && suppression::clippy_allowed(node, source, &measurement.symbol))
    }));
}
fn fragment_measurements(node: Node<'_>, source: Code<'_>) -> Vec<Measurement> {
    let mut cursor = node.walk();
    let Some(tree) = node
        .children(&mut cursor)
        .find(|child| child.kind() == "token_tree")
    else {
        return Vec::new();
    };
    let span = tree.end_byte().saturating_sub(tree.start_byte());
    let empty = span < 2;
    if empty {
        return Vec::new();
    }
    let Some(inner) = source
        .as_str()
        .get(tree.start_byte() + 1..tree.end_byte() - 1)
    else {
        return Vec::new();
    };
    let fragment = ["fn probe(){call(", inner, ")}"].concat();
    let Some(parsed) = reparse(std::path::Path::new("macro.rs"), &fragment) else {
        return Vec::new();
    };
    let root = parsed.root_node();
    let mut scoped = Vec::new();
    scan_fragment(root, Code::new(source.path(), &fragment), &mut scoped);
    for measurement in &mut scoped {
        measurement.line += tree.start_position().row;
    }
    scoped
}
fn scan_fragment(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    match node.kind() {
        "call_expression" => {
            panic_surface::panic_call(node, source, output);
            std_forms::call(node, source, output);
            ownership::call(node, source, output);
        }
        "macro_invocation" => panic_surface::panic_macro(node, source, output),
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        scan_fragment(child, source, output);
    }
}
// DECISION: D085
fn wildcard_use(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let allowed = in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::WildcardImport),
        line: node.start_position().row + 1,
        symbol: "glob".into(),
        actual: 1,
    });
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
fn flag(parameter: Node<'_>, source: Code<'_>) -> bool {
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
