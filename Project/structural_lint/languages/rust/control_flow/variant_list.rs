use crate::structural_lint::languages::neutral_metrics::{
    Code, Measurement, descendants, justified, text,
};
use crate::structural_lint::languages::rust::exemptions::{in_test_context, type_name};
use crate::structural_lint::rule_identities::control_flow::ControlFlow;
use crate::structural_lint::rule_identities::kind::Kind;
use std::iter::successors;
use tree_sitter::Node;

// DECISION: D181
pub(super) fn inspect(array: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(owner) = listed_enum(array, source) else {
        return;
    };
    let allowed = justified(array, source) || in_test_context(array, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::ControlFlow(ControlFlow::VariantList),
        line: array.start_position().row + 1,
        symbol: owner.into(),
        actual: 1,
    });
}
fn listed_enum<'a>(array: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let repeated = array
        .children(&mut array.walk())
        .any(|child| child.kind() == ";");
    let elements: Vec<_> = array
        .named_children(&mut array.walk())
        .filter(|element| !element.is_extra() && element.kind() != "attribute_item")
        .collect();
    let short = repeated || elements.len() < 3;
    if short {
        return None;
    }
    let variants = elements
        .iter()
        .map(|element| variant(*element, source))
        .collect::<Option<Vec<_>>>()?;
    let (owner, _) = *variants.first()?;
    let shared = variants.iter().all(|(name, _)| *name == owner);
    let names: Vec<_> = variants.iter().map(|(_, name)| *name).collect();
    let complete = shared && declared(array, owner, &names, source);
    complete.then_some(owner)
}
fn variant<'a>(element: Node<'_>, source: Code<'a>) -> Option<(&'a str, &'a str)> {
    let scoped = element.kind() == "scoped_identifier";
    let path = element
        .child_by_field_name("path")
        .filter(|path| scoped && path.kind() == "identifier")?;
    let name = text(element.child_by_field_name("name")?, source);
    let owner = match text(path, source) {
        "Self" => successors(path.parent(), Node::parent)
            .find(|node| node.kind() == "impl_item")
            .and_then(|item| item.child_by_field_name("type"))
            .and_then(|implemented| type_name(implemented, source))?,
        named => named,
    };
    Some((owner, name))
}
fn declared(array: Node<'_>, owner: &str, names: &[&str], source: Code<'_>) -> bool {
    successors(Some(array), Node::parent)
        .last()
        .is_some_and(|root| {
            descendants(root)
                .into_iter()
                .filter(|node| node.kind() == "enum_item")
                .filter(|item| {
                    item.child_by_field_name("name")
                        .is_some_and(|name| text(name, source) == owner)
                })
                .any(|item| unit_variants(item, source) == names)
        })
}
fn unit_variants<'a>(item: Node<'_>, source: Code<'a>) -> Vec<&'a str> {
    let Some(body) = item.child_by_field_name("body") else {
        return Vec::new();
    };
    body.named_children(&mut body.walk())
        .filter(|variant| {
            variant.kind() == "enum_variant" && variant.child_by_field_name("body").is_none()
        })
        .filter_map(|variant| variant.child_by_field_name("name"))
        .map(|name| text(name, source))
        .collect()
}
