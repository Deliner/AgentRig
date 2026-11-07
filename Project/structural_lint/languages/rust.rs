use super::test_context::{in_debug_only_context, in_test_context, last_segment};
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
        "unsafe_block" | "impl_item" | "trait_item" | "token_tree" | "token_repetition" => {
            super::unsafe_site::inspect(node, source, output)
        }
        "attribute_item" | "inner_attribute_item" => {
            super::suppression::attribute(node, source, output)
        }
        "static_item" => {
            super::global_state::inspect(node, source, output);
            super::unsafe_site::inspect(node, source, output);
        }
        "macro_invocation" => {
            debug_macro(node, source, output);
            assertion_macro(node, source, output);
            panic_macro(node, source, output);
            macro_body(node, source, output);
            super::global_state::macro_items(node, source, output);
        }
        "call_expression" => call(node, source, output),
        "expression_statement" => discarded_result(node, source, output),
        "scoped_identifier" => super::super_reach::inspect(node, source, output),
        "use_wildcard" => wildcard_use(node, source, output),
        "if_expression" => conditional(node, source, output),
        "match_arm" => super::silent_error::verdict_arm(node, source, output),
        "function_item" | "function_signature_item" | "closure_expression" => {
            declaration(node, source, output);
            super::unsafe_site::inspect(node, source, output);
        }
        _ => {}
    }
}
fn call(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    panic_call(node, source, output);
    eager_fallback(node, source, output);
    disabled_raii(node, source, output);
    super::silent_error::collapsed_read_dir(node, source, output);
    super::recursive_deletion::inspect(node, source, output);
    super::unwind_catch::inspect(node, source, output);
}
fn conditional(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(test) = node.child_by_field_name("condition") else {
        return;
    };
    let binding = test.kind() == "let_condition";
    if binding {
        super::silent_error::verdict_if(node, source, output);
        return;
    }
    condition(node, test, output);
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
    record_signature(node, source, output);
}
// DECISION: D088
// DECISION: D101
fn record_signature(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let named = matches!(node.kind(), "function_item" | "function_signature_item");
    let tuple = annotations(node).any(|annotation| {
        let top = named && annotation.kind() == "tuple_type";
        top || nested_record(annotation, source)
    });
    let plain = !tuple;
    if plain {
        return;
    }
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::TupleAsRecord,
        line: node.start_position().row + 1,
        symbol: "signature".into(),
        actual: 1,
    });
}
fn annotations<'a>(node: Node<'a>) -> impl Iterator<Item = Node<'a>> {
    let parameters = node
        .child_by_field_name("parameters")
        .into_iter()
        .flat_map(|parameters| {
            let mut cursor = parameters.walk();
            parameters
                .named_children(&mut cursor)
                .filter_map(|parameter| parameter.child_by_field_name("type"))
                .collect::<Vec<_>>()
        });
    parameters.chain(node.child_by_field_name("return_type"))
}
fn nested_record(node: Node<'_>, source: &str) -> bool {
    let container = matches!(node.kind(), "generic_type" | "array_type");
    if container {
        return contained_record(node, source);
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| nested_record(child, source))
}
fn contained_record(node: Node<'_>, source: &str) -> bool {
    let record = node.kind() == "tuple_type" && unnamed_record(node, source);
    if record {
        return true;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| contained_record(child, source))
}
fn unnamed_record(tuple: Node<'_>, source: &str) -> bool {
    let mut cursor = tuple.walk();
    tuple.named_child_count() >= 3
        && tuple
            .named_children(&mut cursor)
            .any(|element| scalar(element, source))
}
fn scalar(node: Node<'_>, source: &str) -> bool {
    match node.kind() {
        "primitive_type" => true,
        "type_identifier" => text(node, source) == "String",
        "reference_type" => node
            .child_by_field_name("type")
            .is_some_and(|inner| inner.kind() == "primitive_type"),
        _ => false,
    }
}
// DECISION: D081
fn debug_macro(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(path) = node.child_by_field_name("macro") else {
        return;
    };
    let debug = text(path, source) == "dbg";
    if debug {
        output.push(Measurement {
            kind: Kind::DebugArtifact,
            line: node.start_position().row + 1,
            symbol: "dbg".into(),
            actual: 1,
        });
    }
}
// DECISION: D084
// DECISION: D115
fn assertion_macro(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
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
fn report_assertion(node: Node<'_>, source: &str, symbol: &str, output: &mut Vec<Measurement>) {
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::DebugAssertion,
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
// DECISION: D082
// DECISION: D115
fn panic_call(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
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
    let panic = method && matches!(name, "unwrap" | "expect" | "unwrap_err" | "expect_err")
        || path && name == "resume_unwind";
    if panic {
        report_panic(node, source, name, output);
    }
}
fn panic_macro(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(path) = node.child_by_field_name("macro") else {
        return;
    };
    let name = text(path, source);
    let panic = matches!(name, "panic" | "todo" | "unimplemented" | "unreachable");
    if panic {
        report_panic(node, source, name, output);
    }
}
fn report_panic(node: Node<'_>, source: &str, symbol: &str, output: &mut Vec<Measurement>) {
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::PanicPath,
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
// DECISION: D089
fn eager_fallback(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let Some(field) = function.child_by_field_name("field") else {
        return;
    };
    let method = field.kind() == "field_identifier";
    let name = text(field, source);
    let eager = method && matches!(name, "unwrap_or" | "ok_or" | "map_or");
    let unrelated = !eager;
    if unrelated {
        return;
    }
    let Some(arguments) = node.child_by_field_name("arguments") else {
        return;
    };
    let mut cursor = arguments.walk();
    let propagated = arguments
        .named_children(&mut cursor)
        .find(|argument| !argument.is_extra())
        .is_some_and(propagates);
    let safe = !propagated;
    if safe {
        return;
    }
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::EagerFallback,
        line: node.start_position().row + 1,
        symbol: name.into(),
        actual: 1,
    });
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
// DECISION: D090
// DECISION: D099
// DECISION: D114
fn discarded_result(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let mut cursor = node.walk();
    let Some(inner) = node
        .named_children(&mut cursor)
        .find(|child| !child.is_extra())
    else {
        return;
    };
    let Some(symbol) = discarded_method(inner, source) else {
        return;
    };
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::DiscardedResult,
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn discarded_method<'a>(inner: Node<'_>, source: &'a str) -> Option<&'a str> {
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
// DECISION: D092
fn disabled_raii(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let Some(symbol) = raii_marker(function, source) else {
        return;
    };
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::DisabledRaii,
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn raii_marker<'a>(function: Node<'_>, source: &'a str) -> Option<&'a str> {
    if let Some(field) = function.child_by_field_name("field") {
        let method = field.kind() == "field_identifier";
        let name = text(field, source);
        let consumes = method && matches!(name, "keep" | "into_path");
        return consumes.then_some(name);
    }
    let unscoped = function.kind() != "scoped_identifier";
    if unscoped {
        return None;
    }
    let mut segments = text(function, source).rsplit("::");
    let last = segments.next()?;
    let prior = segments.next()?;
    match (prior, last) {
        ("mem", "forget") => Some("forget"),
        ("ManuallyDrop", "new") => Some("ManuallyDrop"),
        ("Box", "leak") => Some("leak"),
        _ => None,
    }
}
// DECISION: D094
fn macro_body(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let exempt = in_test_context(node, source);
    if exempt {
        return;
    }
    let mut cursor = node.walk();
    let Some(tree) = node
        .children(&mut cursor)
        .find(|child| child.kind() == "token_tree")
    else {
        return;
    };
    let span = tree.end_byte().saturating_sub(tree.start_byte());
    let empty = span < 2;
    if empty {
        return;
    }
    let inner = &source[tree.start_byte() + 1..tree.end_byte() - 1];
    let fragment = ["fn probe(){call(", inner, ")}"].concat();
    let Ok(Some(parsed)) =
        crate::shared::syntax::parse(std::path::Path::new("macro.rs"), &fragment)
    else {
        return;
    };
    let root = parsed.root_node();
    let broken = root.has_error();
    if broken {
        return;
    }
    let base = tree.start_position().row;
    let mut scoped = Vec::new();
    scan_fragment(root, &fragment, &mut scoped);
    for mut measurement in scoped {
        measurement.line += base;
        output.push(measurement);
    }
}
fn scan_fragment(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    match node.kind() {
        "call_expression" => {
            panic_call(node, source, output);
            eager_fallback(node, source, output);
            disabled_raii(node, source, output);
            super::recursive_deletion::inspect(node, source, output);
        }
        "macro_invocation" => panic_macro(node, source, output),
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        scan_fragment(child, source, output);
    }
}
// DECISION: D085
fn wildcard_use(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::WildcardImport,
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
