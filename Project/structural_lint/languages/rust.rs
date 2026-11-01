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
        "attribute_item" | "inner_attribute_item" => attribute(node, source, output),
        "macro_invocation" => {
            debug_macro(node, source, output);
            assertion_macro(node, source, output);
            panic_macro(node, source, output);
        }
        "call_expression" => {
            panic_call(node, source, output);
            eager_fallback(node, source, output);
            disabled_raii(node, source, output);
        }
        "expression_statement" => discarded_result(node, source, output),
        "use_wildcard" => wildcard_use(node, source, output),
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
    let named = matches!(node.kind(), "function_item" | "function_signature_item");
    if named {
        record_signature(node, source, output);
    }
}
// DECISION: D088
fn record_signature(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let tuple = signature_tuple(node);
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
fn signature_tuple(node: Node<'_>) -> bool {
    let parameter = node
        .child_by_field_name("parameters")
        .is_some_and(|parameters| {
            let mut cursor = parameters.walk();
            parameters
                .named_children(&mut cursor)
                .filter_map(|parameter| parameter.child_by_field_name("type"))
                .any(|annotation| annotation.kind() == "tuple_type")
        });
    let returned = node
        .child_by_field_name("return_type")
        .is_some_and(|annotation| annotation.kind() == "tuple_type");
    parameter || returned
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
// DECISION: D074
// DECISION: D080
fn attribute(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(name) = attribute_name(node, source) else {
        return;
    };
    let kind = match name {
        "ignore" => Kind::DisabledTest,
        "allow" => Kind::AllowJustification,
        _ => return,
    };
    let unjustified = !justified(node, source);
    if unjustified {
        output.push(Measurement {
            kind,
            line: node.start_position().row + 1,
            symbol: name.into(),
            actual: 1,
        });
    }
}
fn attribute_name<'a>(node: Node<'_>, source: &'a str) -> Option<&'a str> {
    let attribute = node
        .named_children(&mut node.walk())
        .find(|child| child.kind() == "attribute")?;
    Some(text(attribute.named_child(0)?, source))
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
fn assertion_macro(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(path) = node.child_by_field_name("macro") else {
        return;
    };
    let name = text(path, source);
    let assertion = matches!(name, "debug_assert" | "debug_assert_eq" | "debug_assert_ne");
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
fn panic_call(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let Some(field) = function.child_by_field_name("field") else {
        return;
    };
    let method = field.kind() == "field_identifier";
    let name = text(field, source);
    let panic = method && matches!(name, "unwrap" | "expect");
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
fn discarded_result(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let mut cursor = node.walk();
    let Some(inner) = node
        .named_children(&mut cursor)
        .find(|child| !child.is_extra())
    else {
        return;
    };
    let call = inner.kind() == "call_expression";
    let other = !call;
    if other {
        return;
    }
    let Some(function) = inner.child_by_field_name("function") else {
        return;
    };
    let Some(field) = function.child_by_field_name("field") else {
        return;
    };
    let discarded = field.kind() == "field_identifier" && text(field, source) == "ok";
    let kept = !discarded;
    if kept {
        return;
    }
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::DiscardedResult,
        line: node.start_position().row + 1,
        symbol: "ok".into(),
        actual: 1,
    });
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
fn in_test_context(node: Node<'_>, source: &str) -> bool {
    let mut current = node;
    while let Some(parent) = current.parent() {
        let item = matches!(
            parent.kind(),
            "mod_item" | "function_item" | "function_signature_item"
        );
        let test = item && test_attributed(parent, source);
        if test {
            return true;
        }
        current = parent;
    }
    false
}
fn test_attributed(item: Node<'_>, source: &str) -> bool {
    let mut sibling = item.prev_named_sibling();
    while let Some(node) = sibling {
        let marker = node.kind() == "attribute_item" && test_marker(node, source);
        if marker {
            return true;
        }
        let decoration = matches!(
            node.kind(),
            "attribute_item" | "line_comment" | "block_comment"
        );
        let stop = !decoration;
        if stop {
            return false;
        }
        sibling = node.prev_named_sibling();
    }
    false
}
fn test_marker(node: Node<'_>, source: &str) -> bool {
    let Some(name) = attribute_name(node, source) else {
        return false;
    };
    let cfg_test = name == "cfg" && text(node, source).contains("test");
    name == "test" || cfg_test
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
