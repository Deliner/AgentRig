mod blocking_async;
mod context;
mod control_flow;
mod errors;
mod naming;
mod ownership;
mod panic_surface;
mod signature;
mod super_reach;
mod suppression;
mod test_placement;
mod unsafe_site;
use crate::structural_lint::languages::measure::{
    Analysis, Branch, Code, Measurement, Nest, boolean_expression, boolean_parameters, branching,
    comment, condition, credential, function, justified, mixed_junction, nesting, reparse, text,
};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::ownership::Pair;
use crate::structural_lint::rules::reference::Reference;
use crate::structural_lint::rules::structure::Structure;
use context::in_debug_only_context;
pub(crate) use context::in_test_context;
use tree_sitter::Node;
// DECISION: D017
// DECISION: D037
pub fn inspect(node: Node<'_>, source: Code<'_>, pairs: &[Pair], analysis: &mut Analysis) {
    analysis.mixed_junctions.extend(mixed_junction(node));
    let output = &mut analysis.measurements;
    signature::inspect(node, source, output);
    ownership::inspect(node, source, pairs, output);
    control_flow::inspect(node, source, output);
    errors::inspect(node, source, output);
    naming::inspect(node, source, output);
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
        "expression_statement" | "let_declaration" => discarded_result(node, source, output),
        "scoped_identifier" => super_reach::inspect(node, source, output),
        "mod_item" => test_placement::inspect(node, source, output),
        "use_wildcard" => wildcard_use(node, source, output),
        "if_expression" => conditional(node, output),
        "binary_expression" | "let_chain" => boolean_expression(node, output),
        "function_item" | "function_signature_item" | "closure_expression" => {
            declaration(node, source, output);
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
    eager_fallback(node, source, output);
    panic_surface::catch(node, source, output);
    blocking_async::inspect(node, source, output);
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
fn declaration(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
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
    function(node, source, count, output);
    nesting(node, source, classify, output);
    boolean_parameters(node, source, flag, output);
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
    let allowed = justified(node, source) || in_test_context(node, source);
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
// DECISION: D089
fn eager_fallback(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(field) = node
        .child_by_field_name("function")
        .and_then(|function| function.child_by_field_name("field"))
    else {
        return;
    };
    let method = field.kind() == "field_identifier";
    let name = text(field, source);
    let eager = method && matches!(name, "unwrap_or" | "ok_or" | "map_or" | "or" | "then_some");
    let unrelated = !eager;
    if unrelated {
        return;
    }
    let safe = !evaluated(node, name, source);
    if safe {
        return;
    }
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::EagerFallback),
        line: node.start_position().row + 1,
        symbol: name.into(),
        actual: 1,
    });
}
fn evaluated(call: Node<'_>, name: &str, source: Code<'_>) -> bool {
    let Some(arguments) = call.child_by_field_name("arguments") else {
        return false;
    };
    let mut cursor = arguments.walk();
    let Some(first) = arguments
        .named_children(&mut cursor)
        .find(|argument| !argument.is_extra())
    else {
        return false;
    };
    let called = name == "then_some" && calls(first, source);
    propagates(first) || divides(first) || called
}
fn propagates(node: Node<'_>) -> bool {
    let shortcut = node.kind() == "try_expression";
    if shortcut {
        return true;
    }
    let deferred = node.kind() == "closure_expression";
    if deferred {
        return false;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor).any(propagates)
}
fn divides(node: Node<'_>) -> bool {
    let binary = node.kind() == "binary_expression";
    let operator = node
        .child_by_field_name("operator")
        .map(|operator| operator.kind());
    let divisor = node
        .child_by_field_name("right")
        .map(|divisor| divisor.kind());
    let literal = divisor.is_some_and(|kind| kind.ends_with("_literal"));
    let unchecked = binary && operator == Some("/") && !literal;
    if unchecked {
        return true;
    }
    let deferred = node.kind() == "closure_expression";
    if deferred {
        return false;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor).any(divides)
}
fn calls(node: Node<'_>, source: Code<'_>) -> bool {
    let function = node
        .child_by_field_name("function")
        .filter(|_| node.kind() == "call_expression");
    let constructor = function.is_some_and(|function| {
        let named = matches!(function.kind(), "identifier" | "scoped_identifier");
        named && context::last_segment(function, source).starts_with(char::is_uppercase)
    });
    let expanded = node.kind() == "macro_invocation";
    let computed = function.is_some() && !constructor;
    let invoked = expanded || computed;
    if invoked {
        return true;
    }
    let deferred = node.kind() == "closure_expression";
    if deferred {
        return false;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| calls(child, source))
}
// DECISION: D090
// DECISION: D099
// DECISION: D114
fn discarded_result(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let binding = node.kind() == "let_declaration";
    let symbol = if binding {
        let wildcard = node
            .child_by_field_name("pattern")
            .is_some_and(|pattern| pattern.kind() == "_");
        node.child_by_field_name("value")
            .filter(|value| wildcard && spawned(*value, source))
            .map(|_| "spawn")
    } else {
        let mut cursor = node.walk();
        let inner = node
            .named_children(&mut cursor)
            .find(|child| !child.is_extra());
        inner.and_then(|inner| {
            discarded_method(inner, source).or_else(|| spawned(inner, source).then_some("spawn"))
        })
    };
    let Some(symbol) = symbol else {
        return;
    };
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::DiscardedResult),
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn discarded_method<'a>(inner: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let propagated = inner.kind() == "try_expression";
    let call = if propagated {
        inner.named_child(0)?
    } else {
        inner
    };
    let other = call.kind() != "call_expression";
    if other {
        return None;
    }
    let field = call
        .child_by_field_name("function")?
        .child_by_field_name("field")?;
    let unnamed = field.kind() != "field_identifier";
    if unnamed {
        return None;
    }
    let name = text(field, source);
    let dropped = if propagated {
        let nullary = call.child_by_field_name("arguments")?.named_child_count() == 0;
        ["status", "output"].contains(&name) && nullary
    } else {
        ["ok", "err", "map", "and_then", "map_or", "map_or_else"].contains(&name)
    };
    dropped.then_some(name)
}
// DECISION: D155
fn spawned(value: Node<'_>, source: Code<'_>) -> bool {
    let mut current = value;
    loop {
        let propagated = current.kind() == "try_expression";
        if propagated {
            let Some(inner) = current.named_child(0) else {
                return false;
            };
            current = inner;
            continue;
        }
        let Some(function) = current
            .child_by_field_name("function")
            .filter(|_| current.kind() == "call_expression")
        else {
            return false;
        };
        let name = match function.kind() {
            "identifier" => text(function, source),
            "scoped_identifier" | "field_expression" => function
                .child_by_field_name("name")
                .or_else(|| function.child_by_field_name("field"))
                .map_or("", |name| text(name, source)),
            _ => "",
        };
        let unwrapped =
            matches!(name, "unwrap" | "expect") && function.kind() == "field_expression";
        let Some(receiver) = function.child_by_field_name("value").filter(|_| unwrapped) else {
            return name == "spawn";
        };
        current = receiver;
    }
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
    scan_fragment(root, Code::new(&fragment), &mut scoped);
    for measurement in &mut scoped {
        measurement.line += tree.start_position().row;
    }
    scoped
}
fn scan_fragment(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    match node.kind() {
        "call_expression" => {
            panic_surface::panic_call(node, source, output);
            eager_fallback(node, source, output);
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
    let allowed = justified(node, source) || in_test_context(node, source);
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
