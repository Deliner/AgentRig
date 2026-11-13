use super::{method, sole_argument};
use crate::structural_lint::languages::neutral_metrics::{Code, text};
use tree_sitter::Node;

// DECISION: D181
pub(super) fn inspect(call: Node<'_>, source: Code<'_>) -> Option<&'static str> {
    let (name, _) = method(call, source)?;
    let symbol = match name {
        "with_extension" => "with_extension",
        "set_extension" => "set_extension",
        _ => return None,
    };
    let argument = sole_argument(call)?;
    let literal = matches!(argument.kind(), "string_literal" | "raw_string_literal");
    let dotted = literal
        && argument
            .named_child(0)
            .filter(|content| content.kind() == "string_content")
            .is_some_and(|content| text(content, source).starts_with('.'));
    dotted.then_some(symbol)
}
