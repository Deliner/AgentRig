use super::{method, sole_argument};
use crate::structural_lint::languages::neutral_metrics::Code;
use crate::structural_lint::languages::rust::exemptions::zero_literal;
use tree_sitter::Node;

pub(super) fn inspect(call: Node<'_>, source: Code<'_>) -> Option<&'static str> {
    let (name, _) = method(call, source)?;
    let argument = sole_argument(call).filter(|_| name == "remove")?;
    let integer = argument.kind() == "integer_literal";
    let front = integer && zero_literal(argument, source);
    front.then_some("remove(0)")
}
