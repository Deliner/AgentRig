use super::checked_constructor::{Owner, owned};
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, descendants, text};
use crate::structural_lint::languages::rust::exemptions::{
    attributed, derives, in_test_context, justified_item, last_segment, listed, type_name,
};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::ownership::Ownership;
use crate::structural_lint::rule_identities::signature::Signature;
use std::collections::BTreeSet;
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
pub(super) fn secret_debug(definition: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let exempt = in_test_context(definition, source) || !derives(definition, source, &["Debug"]);
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
                kind: Kind::Signature(Signature::SecretDebug),
                line: name.start_position().row + 1,
                symbol: format!("{owner}.{field_name}"),
                actual: 1,
            });
        }
    }
}
// DECISION: D131
pub(super) fn unchecked_deserialize(
    root: Node<'_>,
    source: Code<'_>,
    output: &mut Vec<Measurement>,
) {
    let nodes = descendants(root);
    let implementations: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| node.kind() == "impl_item")
        .collect();
    let definitions = nodes
        .into_iter()
        .filter(|node| node.kind() == "struct_item");
    for definition in definitions {
        let Some(symbol) = bypass(definition, &implementations, source) else {
            continue;
        };
        output.push(Measurement {
            kind: Kind::Signature(Signature::UncheckedDeserialize),
            line: definition.start_position().row + 1,
            symbol,
            actual: 1,
        });
    }
}
fn bypass(definition: Node<'_>, implementations: &[Node<'_>], source: Code<'_>) -> Option<String> {
    let derived = derives(definition, source, &["Deserialize"]);
    let routed = attributed(definition, source, converts) || in_test_context(definition, source);
    (derived && !routed).then_some(())?;
    let name = text(definition.child_by_field_name("name")?, source);
    let owner = Owner {
        name,
        plain: plain_fields(definition.child_by_field_name("body")?, source),
        source,
    };
    let manual = implementations.iter().any(|implementation| {
        let contract = implementation.child_by_field_name("trait");
        owned(*implementation, name, source)
            && contract.and_then(|path| type_name(path, source)) == Some("Deserialize")
    });
    (!manual).then_some(())?;
    owner.checked(implementations)
}
// DECISION: D156
pub(super) fn open_checked_field(root: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let nodes = descendants(root);
    let implementations: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| node.kind() == "impl_item")
        .collect();
    for definition in nodes.iter().filter(|node| node.kind() == "struct_item") {
        let Some(symbol) = exposed(*definition, &implementations, source) else {
            continue;
        };
        output.push(Measurement {
            kind: Kind::Ownership(Ownership::OpenCheckedField),
            line: definition.start_position().row + 1,
            symbol,
            actual: 1,
        });
    }
}
fn exposed(definition: Node<'_>, implementations: &[Node<'_>], source: Code<'_>) -> Option<String> {
    let skipped = in_test_context(definition, source) || justified_item(definition, source);
    (!skipped).then_some(())?;
    let owner = Owner {
        name: text(definition.child_by_field_name("name")?, source),
        plain: visible_fields(definition.child_by_field_name("body")?, source),
        source,
    };
    (!owner.plain.is_empty()).then_some(())?;
    owner.checked(implementations)
}
fn plain_fields(body: Node<'_>, source: Code<'_>) -> BTreeSet<String> {
    let mut plain = BTreeSet::new();
    let mut routed = false;
    let mut position = 0;
    let mut cursor = body.walk();
    for child in body.named_children(&mut cursor) {
        match child.kind() {
            "attribute_item" => routed |= routes_field(child, source),
            "visibility_modifier" | "line_comment" | "block_comment" => {}
            kind => {
                let named = child
                    .child_by_field_name("name")
                    .filter(|_| kind == "field_declaration");
                let field = named.map_or_else(
                    || position.to_string(),
                    |name| text(name, source).to_owned(),
                );
                position += 1;
                let unrouted = !std::mem::take(&mut routed);
                if unrouted {
                    plain.insert(field);
                }
            }
        }
    }
    plain
}
fn visible_fields(body: Node<'_>, source: Code<'_>) -> BTreeSet<String> {
    let mut visible = BTreeSet::new();
    let mut exposed = false;
    let mut position = 0;
    let mut cursor = body.walk();
    for child in body.named_children(&mut cursor) {
        match child.kind() {
            "visibility_modifier" => exposed = true,
            "attribute_item" | "line_comment" | "block_comment" => {}
            "field_declaration" => {
                let mut parts = child.walk();
                let public = child
                    .named_children(&mut parts)
                    .any(|part| part.kind() == "visibility_modifier");
                let name = child.child_by_field_name("name").filter(|_| public);
                visible.extend(name.map(|name| text(name, source).to_owned()));
            }
            _ => {
                let public = std::mem::take(&mut exposed);
                visible.extend(public.then(|| position.to_string()));
                position += 1;
            }
        }
    }
    visible
}
fn converts(attribute_item: Node<'_>, source: Code<'_>) -> bool {
    listed(attribute_item, source, "serde", &["try_from", "from"])
}
fn routes_field(attribute_item: Node<'_>, source: Code<'_>) -> bool {
    listed(
        attribute_item,
        source,
        "serde",
        &["deserialize_with", "with"],
    )
}
fn printable(declared: Node<'_>, source: Code<'_>) -> bool {
    let optional = generic_argument(declared, source, "Option");
    plain(declared, source) || optional.is_some_and(|inner| plain(inner, source))
}
fn plain(declared: Node<'_>, source: Code<'_>) -> bool {
    match declared.kind() {
        "reference_type" => declared
            .child_by_field_name("type")
            .is_some_and(|target| primitive(target, source, "str")),
        "generic_type" => generic_argument(declared, source, "Vec")
            .is_some_and(|element| primitive(element, source, "u8")),
        _ => last_segment(declared, source) == "String",
    }
}
fn generic_argument<'a>(declared: Node<'a>, source: Code<'_>, base: &str) -> Option<Node<'a>> {
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
fn primitive(node: Node<'_>, source: Code<'_>, name: &str) -> bool {
    node.kind() == "primitive_type" && text(node, source) == name
}
