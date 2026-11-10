use crate::structural_lint::languages::measure::{Code, Measurement, text};
use crate::structural_lint::languages::rust::context::{derives, in_test_context, justified_item};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use tree_sitter::Node;

// DECISION: D155
pub(super) fn inspect(definition: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(body) = definition.child_by_field_name("body") else {
        return;
    };
    let mut cursor = body.walk();
    let variants: Vec<_> = body
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "enum_variant")
        .collect();
    let implicit = variants.len() > 1
        && variants
            .iter()
            .any(|variant| variant.child_by_field_name("value").is_none());
    let ordered = implicit && derives(definition, source, &["PartialOrd", "Ord"]);
    let exempt =
        !ordered || in_test_context(definition, source) || justified_item(definition, source);
    if exempt {
        return;
    }
    output.push(Measurement {
        kind: Kind::Signature(Signature::ImplicitOrder),
        line: definition.start_position().row + 1,
        symbol: definition
            .child_by_field_name("name")
            .map_or("", |name| text(name, source))
            .into(),
        actual: 1,
    });
}
