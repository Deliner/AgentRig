use crate::structural_lint::languages::neutral_metrics::{Code, text};
use tree_sitter::Node;

pub(super) fn method_receiver<'a>(
    call: Node<'a>,
    method: &str,
    source: Code<'_>,
) -> Option<Node<'a>> {
    let bare = call.kind() == "call_expression"
        && call
            .child_by_field_name("arguments")
            .is_some_and(|arguments| arguments.named_child_count() == 0);
    let function = bare
        .then(|| call.child_by_field_name("function"))
        .flatten()?;
    let named = function
        .child_by_field_name("field")
        .is_some_and(|field| text(field, source) == method);
    named
        .then(|| function.child_by_field_name("value"))
        .flatten()
}
