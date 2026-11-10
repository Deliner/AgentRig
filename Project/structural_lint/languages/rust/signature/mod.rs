mod derived_trait;
mod ord_equality;
mod tuple_record;
use crate::structural_lint::languages::measure::Measurement;
use tree_sitter::Node;
pub(super) fn inspect(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    match node.kind() {
        "source_file" => {
            ord_equality::inspect(node, source, output);
            derived_trait::unchecked_deserialize(node, source, output);
        }
        "struct_item" => derived_trait::secret_debug(node, source, output),
        "function_item" | "function_signature_item" | "closure_expression" => {
            tuple_record::record_signature(node, source, output);
        }
        _ => {}
    }
}
