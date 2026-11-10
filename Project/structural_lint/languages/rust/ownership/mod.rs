mod disabled_raii;
mod global_state;
mod guard_await;
mod paired_calls;
mod recursive_deletion;
mod unit_lock;
mod wrapper;
use crate::structural_lint::languages::measure::Measurement;
use crate::structural_lint::rules::ownership::Pair;
use tree_sitter::Node;
pub(super) fn inspect(node: Node<'_>, source: &str, pairs: &[Pair], output: &mut Vec<Measurement>) {
    match node.kind() {
        "source_file" => wrapper::inspect(node, source, output),
        "static_item" => global_state::inspect(node, source, output),
        "macro_invocation" => global_state::macro_items(node, source, output),
        "call_expression" => call(node, source, output),
        "let_declaration" => guard_await::inspect(node, source, output),
        "generic_type" => unit_lock::declared(node, source, output),
        "block" => paired_calls::inspect(node, source, pairs, output),
        _ => {}
    }
}
pub(super) fn call(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    disabled_raii::inspect(node, source, output);
    recursive_deletion::inspect(node, source, output);
    unit_lock::constructed(node, source, output);
}
