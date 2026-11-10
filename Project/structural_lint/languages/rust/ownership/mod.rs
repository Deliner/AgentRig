mod disabled_raii;
mod global_state;
mod guard_await;
mod recursive_deletion;
mod unit_lock;
use crate::structural_lint::languages::measure::Measurement;
use tree_sitter::Node;
pub(super) fn inspect(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    match node.kind() {
        "static_item" => global_state::inspect(node, source, output),
        "macro_invocation" => global_state::macro_items(node, source, output),
        "call_expression" => call(node, source, output),
        "let_declaration" => guard_await::inspect(node, source, output),
        "generic_type" => unit_lock::declared(node, source, output),
        _ => {}
    }
}
pub(super) fn call(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    disabled_raii::inspect(node, source, output);
    recursive_deletion::inspect(node, source, output);
    unit_lock::constructed(node, source, output);
}
