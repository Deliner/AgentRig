use crate::structural_lint::languages::measure::{Code, Measurement, descendants, text};
use crate::structural_lint::languages::rust::context::{
    in_test_context, justified_item, type_name,
};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::ownership::Ownership;
use tree_sitter::Node;

const GUARDS: &[&str] = &[
    "MutexGuard",
    "RwLockReadGuard",
    "RwLockWriteGuard",
    "Ref",
    "RefMut",
];

// DECISION: D166
pub(super) fn inspect(function: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let mut cursor = function.walk();
    let visible = function
        .children(&mut cursor)
        .any(|child| child.kind() == "visibility_modifier");
    let Some(returned) = function
        .child_by_field_name("return_type")
        .filter(|_| visible)
    else {
        return;
    };
    let guarded = descendants(returned).into_iter().any(|node| {
        node.kind() == "generic_type"
            && type_name(node, source).is_some_and(|name| GUARDS.contains(&name))
    });
    let allowed = !guarded || in_test_context(function, source) || justified_item(function, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Ownership(Ownership::ExposedGuard),
        line: function.start_position().row + 1,
        symbol: function
            .child_by_field_name("name")
            .map_or("", |name| text(name, source))
            .into(),
        actual: 1,
    });
}
