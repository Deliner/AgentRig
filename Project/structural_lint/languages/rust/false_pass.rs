use super::blocking_async::sleeping;
use super::call_fragment;
use super::context::{
    attribute_path, attributed, cfg_excludes, in_test_context, last_segment, test_attribute,
};
use crate::structural_lint::languages::measure::{
    Code, Measurement, descendants, justified, reparse, text,
};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use std::path::Path;
use tree_sitter::Node;

pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    match node.kind() {
        "attribute_item" => {
            untargeted_panic(node, source, output);
            test_substitution(node, source, output);
        }
        "inner_attribute_item" => test_substitution(node, source, output),
        "macro_invocation" => untargeted_error(node, source, output),
        "call_expression" => sleep_in_test(node, source, output),
        _ => {}
    }
}
fn untargeted_panic(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(path) = attribute_path(node) else {
        return;
    };
    let other = last_segment(path, source) != "should_panic";
    if other {
        return;
    }
    let Some(attribute) = path.parent() else {
        return;
    };
    let valued = attribute.child_by_field_name("value").is_some();
    let expected = attribute
        .child_by_field_name("arguments")
        .is_some_and(|arguments| names_expected(arguments, source));
    let targeted = valued || expected;
    let allowed = targeted || justified(node, source);
    if allowed {
        return;
    }
    report(node, Reference::UntargetedPanic, "should_panic", output);
}
fn names_expected(arguments: Node<'_>, source: Code<'_>) -> bool {
    descendants(arguments)
        .into_iter()
        .any(|token| token.kind() == "identifier" && text(token, source) == "expected")
}
fn test_substitution(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let item = node
        .parent()
        .is_some_and(|parent| matches!(parent.kind(), "source_file" | "declaration_list"));
    let substituted = item && cfg_excludes(node, source, "test");
    let other = !substituted;
    if other {
        return;
    }
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    report(node, Reference::TestSubstitution, "cfg", output);
}
fn untargeted_error(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let assertion = node
        .child_by_field_name("macro")
        .is_some_and(|path| text(path, source) == "assert");
    let tested = assertion && in_test_context(node, source);
    let candidate = tested && !justified(node, source);
    let reported = candidate && checks_error(node, source);
    if reported {
        report(node, Reference::UntargetedError, "is_err", output);
    }
}
fn checks_error(node: Node<'_>, source: Code<'_>) -> bool {
    let mut cursor = node.walk();
    let Some(tree) = node
        .children(&mut cursor)
        .find(|child| child.kind() == "token_tree")
    else {
        return false;
    };
    let Some(fragment) = call_fragment(tree, source) else {
        return false;
    };
    let Some(parsed) = reparse(Path::new("macro.rs"), &fragment) else {
        return false;
    };
    let code = Code::new(&fragment);
    first_argument(parsed.root_node()).is_some_and(|condition| bare_is_err(condition, code))
}
fn first_argument(root: Node<'_>) -> Option<Node<'_>> {
    let call = root
        .named_child(0)?
        .child_by_field_name("body")?
        .named_child(0)?;
    let arguments = call.child_by_field_name("arguments")?;
    let mut cursor = arguments.walk();
    arguments
        .named_children(&mut cursor)
        .find(|argument| !argument.is_extra())
}
fn bare_is_err(condition: Node<'_>, source: Code<'_>) -> bool {
    let named = condition
        .child_by_field_name("function")
        .filter(|function| function.kind() == "field_expression")
        .and_then(|function| function.child_by_field_name("field"))
        .is_some_and(|field| text(field, source) == "is_err");
    let bare = condition
        .child_by_field_name("arguments")
        .is_some_and(|arguments| arguments.named_child_count() == 0);
    named && bare
}
fn sleep_in_test(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let sleep = node
        .child_by_field_name("function")
        .is_some_and(|function| sleeping(function, source));
    let tested = sleep && in_test_function(node, source);
    let reported = tested && !justified(node, source);
    if reported {
        report(node, Reference::SleepInTest, "sleep", output);
    }
}
fn in_test_function(node: Node<'_>, source: Code<'_>) -> bool {
    std::iter::successors(node.parent(), Node::parent)
        .find(|parent| parent.kind() == "function_item")
        .is_some_and(|function| attributed(function, source, test_attribute))
}
fn report(node: Node<'_>, reference: Reference, symbol: &str, output: &mut Vec<Measurement>) {
    output.push(Measurement {
        kind: Kind::Reference(reference),
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
