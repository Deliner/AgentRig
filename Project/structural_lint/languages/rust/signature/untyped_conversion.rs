use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::context::{in_test_context, last_segment};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use tree_sitter::Node;

const CONVERSIONS: [&str; 6] = ["parse", "into", "try_into", "sum", "product", "collect"];

pub(super) fn inspect(binding: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let typed = binding.child_by_field_name("type").is_some();
    let plain = binding
        .child_by_field_name("pattern")
        .is_some_and(|pattern| pattern.kind() == "identifier");
    let value = binding
        .child_by_field_name("value")
        .filter(|_| plain && !typed);
    let Some(symbol) = value.and_then(|value| conversion(unwrapped(value), source)) else {
        return;
    };
    let allowed = justified(binding, source) || in_test_context(binding, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Signature(Signature::UntypedConversion),
        line: binding.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn unwrapped(node: Node<'_>) -> Node<'_> {
    let propagated = node.kind() == "try_expression";
    match node.named_child(0) {
        Some(inner) if propagated => unwrapped(inner),
        _ => node,
    }
}
fn conversion<'a>(call: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let invoked = call.kind() == "call_expression";
    let function = call.child_by_field_name("function").filter(|_| invoked)?;
    let empty = call.child_by_field_name("arguments")?.named_child_count() == 0;
    match function.kind() {
        "field_expression" => {
            let name = text(function.child_by_field_name("field")?, source);
            let converting = empty && CONVERSIONS.contains(&name);
            converting.then_some(name)
        }
        "scoped_identifier" => {
            let owner = last_segment(function.child_by_field_name("path")?, source);
            let name = text(function.child_by_field_name("name")?, source);
            let defaulted = empty && owner == "Default" && name == "default";
            let converted = owner == "From" && name == "from";
            let generic = defaulted || converted;
            generic.then_some(name)
        }
        _ => None,
    }
}
