use crate::structural_lint::languages::measure::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::context::{
    in_test_context, justified_item, trait_impl_member, type_name,
};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use tree_sitter::Node;

const SCALARS: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "char",
];
const CLIPPY_LOOKS_THROUGH: &[&str] = &[
    "generic_type",
    "type_arguments",
    "reference_type",
    "pointer_type",
    "array_type",
    "tuple_type",
    "scoped_type_identifier",
    "bracketed_type",
    "qualified_type",
    "parameter",
    "parameters",
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
// DECISION: D162
// DECISION: D163
// DECISION: D164
pub(super) fn option(generic: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(argument) = option_argument(generic, source) else {
        return;
    };
    let flag = argument.kind() == "primitive_type" && text(argument, source) == "bool";
    let reported_elsewhere = clippy_checks(generic) || trait_sets_signature(generic);
    let nested = option_argument(argument, source).is_some()
        && !reported_elsewhere
        && option_argument_of_parent(generic, source).is_none();
    let reported = flag || nested;
    let allowed = !reported || in_test_context(generic, source) || justified(generic, source);
    if allowed {
        return;
    }
    push(generic, source, Signature::AmbiguousOption, output);
}
fn option_argument<'a>(node: Node<'a>, source: Code<'_>) -> Option<Node<'a>> {
    let optional = type_name(node, source) == Some("Option");
    node.child_by_field_name("type_arguments")
        .filter(|_| optional)
        .and_then(single_argument)
}
fn option_argument_of_parent<'a>(node: Node<'a>, source: Code<'_>) -> Option<Node<'a>> {
    let outer = node.parent()?.parent()?;
    option_argument(outer, source).filter(|argument| *argument == node)
}
// DECISION: D163
fn clippy_checks(generic: Node<'_>) -> bool {
    let mut node = generic;
    while let Some(parent) = node.parent() {
        match parent.kind() {
            kind if CLIPPY_LOOKS_THROUGH.contains(&kind) => node = parent,
            "field_declaration" | "ordered_field_declaration_list" => return true,
            "function_item" | "function_signature_item" | "const_item" | "static_item" => {
                return checked_member(parent);
            }
            _ => return false,
        }
    }
    false
}
fn checked_member(item: Node<'_>) -> bool {
    let extern_member = item
        .parent()
        .and_then(|list| list.parent())
        .is_some_and(|owner| owner.kind() == "foreign_mod_item");
    !(trait_impl_member(item) || extern_member)
}
// DECISION: D164
fn trait_sets_signature(generic: Node<'_>) -> bool {
    let mut node = generic;
    while let Some(parent) = node.parent() {
        let Some(fields) = signature_fields(parent.kind()) else {
            node = parent;
            continue;
        };
        let declared = fields
            .iter()
            .any(|field| parent.child_by_field_name(field) == Some(node));
        return declared && trait_impl_member(parent);
    }
    false
}
fn signature_fields(kind: &str) -> Option<&'static [&'static str]> {
    match kind {
        "function_item" => Some(&["parameters", "return_type"]),
        "const_item" => Some(&["type"]),
        _ => None,
    }
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
