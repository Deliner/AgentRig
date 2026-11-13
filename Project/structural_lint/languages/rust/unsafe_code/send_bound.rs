use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, text};
use crate::structural_lint::languages::rust::exemptions::{
    in_test_context, justified_item, plain_name, type_name,
};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use tree_sitter::Node;

pub(super) fn inspect(item: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(marker) = unbounded(item, source) else {
        return;
    };
    let allowed = justified_item(item, source) || in_test_context(item, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::UnboundedSend),
        line: item.start_position().row + 1,
        symbol: marker.into(),
        actual: 1,
    });
}
fn unbounded<'a>(item: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let unsafe_impl = item
        .children(&mut item.walk())
        .any(|child| child.kind() == "unsafe");
    let marker = item
        .child_by_field_name("trait")
        .filter(|_| unsafe_impl)
        .and_then(|name| plain_name(name, source))
        .filter(|name| matches!(*name, "Send" | "Sync"))?;
    let parameters = item.child_by_field_name("type_parameters")?;
    let predicates: Vec<_> = item
        .children(&mut item.walk())
        .filter(|child| child.kind() == "where_clause")
        .flat_map(|clause| {
            clause
                .named_children(&mut clause.walk())
                .collect::<Vec<_>>()
        })
        .collect();
    let open = parameters
        .named_children(&mut parameters.walk())
        .filter(|parameter| parameter.kind() == "type_parameter")
        .any(|parameter| !bounded(parameter, &predicates, source));
    open.then_some(marker)
}
fn bounded(parameter: Node<'_>, predicates: &[Node<'_>], source: Code<'_>) -> bool {
    let Some(name) = parameter.child_by_field_name("name") else {
        return true;
    };
    let listed = predicates
        .iter()
        .filter(|predicate| {
            predicate
                .child_by_field_name("left")
                .is_some_and(|left| text(left, source) == text(name, source))
        })
        .filter_map(|predicate| predicate.child_by_field_name("bounds"));
    parameter
        .child_by_field_name("bounds")
        .into_iter()
        .chain(listed)
        .any(|bounds| threaded(bounds, source))
}
fn threaded(bounds: Node<'_>, source: Code<'_>) -> bool {
    bounds
        .named_children(&mut bounds.walk())
        .any(|bound| matches!(type_name(bound, source), Some("Send" | "Sync")))
}
