use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::context::{
    in_test_context, justified_item, type_name,
};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use tree_sitter::Node;

const SCALARS: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "char",
];

// DECISION: D156
pub(super) fn alias(item: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let generic = item.child_by_field_name("type_parameters").is_some();
    let Some(aliased) = item.child_by_field_name("type").filter(|_| !generic) else {
        return;
    };
    let primitive = match aliased.kind() {
        "primitive_type" => SCALARS.contains(&text(aliased, source)),
        "type_identifier" => text(aliased, source) == "String",
        _ => false,
    };
    let allowed = !primitive || in_test_context(item, source) || justified_item(item, source);
    if allowed {
        return;
    }
    push(item, source, Signature::PrimitiveAlias, output);
}
// DECISION: D156
pub(super) fn option(generic: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let optional = type_name(generic, source) == Some("Option");
    let Some(argument) = generic
        .child_by_field_name("type_arguments")
        .filter(|_| optional)
        .and_then(single_argument)
    else {
        return;
    };
    let flag = argument.kind() == "primitive_type" && text(argument, source) == "bool";
    let nested = type_name(argument, source) == Some("Option");
    let ambiguous = flag || nested;
    let allowed = !ambiguous || in_test_context(generic, source) || justified(generic, source);
    if allowed {
        return;
    }
    push(generic, source, Signature::AmbiguousOption, output);
}
fn push(node: Node<'_>, source: Code<'_>, kind: Signature, output: &mut Vec<Measurement>) {
    output.push(Measurement {
        kind: Kind::Signature(kind),
        line: node.start_position().row + 1,
        symbol: node
            .child_by_field_name("name")
            .map_or_else(|| text(node, source), |name| text(name, source))
            .into(),
        actual: 1,
    });
}
fn single_argument(arguments: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = arguments.walk();
    let types: Vec<_> = arguments
        .named_children(&mut cursor)
        .filter(|argument| !argument.is_extra() && argument.kind() != "lifetime")
        .collect();
    let [argument] = types.as_slice() else {
        return None;
    };
    Some(*argument)
}
