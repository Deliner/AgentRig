use crate::structural_lint::languages::measure::{Measurement, descendants, text};
use crate::structural_lint::languages::rust::context::{in_test_context, justified_item};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use tree_sitter::Node;

// DECISION: D156
pub(super) fn inspect(definition: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(list) = definition.child_by_field_name("body") else {
        return;
    };
    let mut cursor = list.walk();
    let defaults = list
        .named_children(&mut cursor)
        .filter(|item| item.kind() == "function_item");
    for function in defaults {
        let Some(body) = function.child_by_field_name("body") else {
            continue;
        };
        let mut statements = body.walk();
        let empty = !body
            .named_children(&mut statements)
            .any(|statement| !statement.is_extra());
        let receiving = descendants(body)
            .into_iter()
            .any(|node| node.kind() == "self");
        let trivial = empty || receiving;
        let allowed =
            trivial || in_test_context(function, source) || justified_item(function, source);
        if allowed {
            continue;
        }
        output.push(Measurement {
            kind: Kind::Signature(Signature::ConstantDefault),
            line: function.start_position().row + 1,
            symbol: function
                .child_by_field_name("name")
                .map_or("", |name| text(name, source))
                .into(),
            actual: 1,
        });
    }
}
