use super::context::{attributed, in_test_context, test_marker};
use crate::structural_lint::languages::measure::{Measurement, text};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::structure::Structure;
use tree_sitter::Node;

// DECISION: D145
pub(super) fn inspect(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(name) = node.child_by_field_name("name") else {
        return;
    };
    let inline = node.child_by_field_name("body").is_some();
    if inline {
        return;
    }
    let gated = attributed(node, source, test_marker) || in_test_context(node, source);
    let production = !gated;
    if production {
        return;
    }
    let name = text(name, source);
    let declared = declared_path(node, source);
    let location = declared.map_or(vec![name], |path| path.split('/').collect());
    let placed = ancestors(node, source)
        .chain(location)
        .any(|part| matches!(part, "tests" | "tests.rs"));
    if placed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Structure(Structure::TestPlacement),
        line: node.start_position().row + 1,
        symbol: declared.unwrap_or(name).into(),
        actual: 1,
    });
}
fn ancestors<'a>(node: Node<'a>, source: &'a str) -> impl Iterator<Item = &'a str> {
    std::iter::successors(node.parent(), Node::parent)
        .filter(|parent| parent.kind() == "mod_item")
        .filter_map(|parent| parent.child_by_field_name("name"))
        .map(move |name| text(name, source))
}
fn declared_path<'a>(node: Node<'_>, source: &'a str) -> Option<&'a str> {
    std::iter::successors(node.prev_named_sibling(), Node::prev_named_sibling)
        .take_while(|item| item.is_extra() || item.kind() == "attribute_item")
        .filter_map(|item| item.named_child(0))
        .filter(|attribute| attribute.kind() == "attribute")
        .find(|attribute| {
            attribute
                .named_child(0)
                .is_some_and(|path| text(path, source) == "path")
        })
        .and_then(|attribute| attribute.child_by_field_name("value"))
        .and_then(|value| value.named_child(0))
        .map(|content| text(content, source))
}
