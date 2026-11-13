use super::method;
use crate::structural_lint::languages::neutral_metrics::{Code, descendants, text};
use tree_sitter::Node;

pub(super) fn inspect(call: Node<'_>, source: Code<'_>) -> Option<&'static str> {
    let (name, receiver) = method(call, source)?;
    let symbol = match name {
        "unwrap_or" => "unwrap_or",
        "unwrap_or_else" => "unwrap_or_else",
        _ => return None,
    };
    let compared = method(receiver, source).is_some_and(|(inner, _)| inner == "partial_cmp");
    let ordered = descendants(call.child_by_field_name("arguments")?)
        .into_iter()
        .any(|node| {
            node.kind() == "identifier"
                && matches!(text(node, source), "Equal" | "Less" | "Greater")
        });
    let fallback = compared && ordered;
    fallback.then_some(symbol)
}
