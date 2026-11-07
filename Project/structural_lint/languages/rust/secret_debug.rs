use super::context::{attributed, in_test_context, last_segment};
use crate::structural_lint::languages::measure::{Measurement, descendants, text};
use crate::structural_lint::rules::kind::Kind;
use tree_sitter::Node;

const SECRET_NAMES: &[&str] = &[
    "password",
    "passwd",
    "secret",
    "token",
    "api_key",
    "apikey",
    "private_key",
    "access_token",
    "refresh_token",
    "client_secret",
    "jwt",
    "cvv",
];

// DECISION: D130
pub(super) fn inspect(definition: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let exempt =
        in_test_context(definition, source) || !attributed(definition, source, derives_debug);
    if exempt {
        return;
    }
    let Some(body) = definition
        .child_by_field_name("body")
        .filter(|body| body.kind() == "field_declaration_list")
    else {
        return;
    };
    let owner = definition
        .child_by_field_name("name")
        .map_or("", |name| text(name, source));
    let mut cursor = body.walk();
    for field in body.named_children(&mut cursor) {
        let (Some(name), Some(declared)) = (
            field.child_by_field_name("name"),
            field.child_by_field_name("type"),
        ) else {
            continue;
        };
        let field_name = text(name, source);
        let secret = SECRET_NAMES.contains(&field_name) && printable(declared, source);
        if secret {
            output.push(Measurement {
                kind: Kind::SecretDebug,
                line: name.start_position().row + 1,
                symbol: format!("{owner}.{field_name}"),
                actual: 1,
            });
        }
    }
}
fn derives_debug(attribute_item: Node<'_>, source: &str) -> bool {
    descendants(attribute_item).into_iter().any(|node| {
        let derive = node.kind() == "identifier" && text(node, source) == "derive";
        derive
            && node
                .next_named_sibling()
                .filter(|list| list.kind() == "token_tree")
                .is_some_and(|list| names(list, source, "Debug"))
    })
}
fn names(list: Node<'_>, source: &str, wanted: &str) -> bool {
    descendants(list)
        .into_iter()
        .any(|token| token.kind() == "identifier" && text(token, source) == wanted)
}
fn printable(declared: Node<'_>, source: &str) -> bool {
    let optional = generic_argument(declared, source, "Option");
    plain(declared, source) || optional.is_some_and(|inner| plain(inner, source))
}
fn plain(declared: Node<'_>, source: &str) -> bool {
    match declared.kind() {
        "reference_type" => declared
            .child_by_field_name("type")
            .is_some_and(|target| primitive(target, source, "str")),
        "generic_type" => generic_argument(declared, source, "Vec")
            .is_some_and(|element| primitive(element, source, "u8")),
        _ => last_segment(declared, source) == "String",
    }
}
fn generic_argument<'a>(declared: Node<'a>, source: &str, base: &str) -> Option<Node<'a>> {
    let named = declared.kind() == "generic_type"
        && declared
            .child_by_field_name("type")
            .is_some_and(|path| last_segment(path, source) == base);
    named.then_some(())?;
    let arguments = declared.child_by_field_name("type_arguments")?;
    let mut cursor = arguments.walk();
    let types: Vec<_> = arguments.named_children(&mut cursor).collect();
    match types.as_slice() {
        [single] => Some(*single),
        _ => None,
    }
}
fn primitive(node: Node<'_>, source: &str, name: &str) -> bool {
    node.kind() == "primitive_type" && text(node, source) == name
}
