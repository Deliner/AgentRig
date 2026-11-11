mod checked_constructor;
mod constant_default;
mod derived_trait;
mod implicit_order;
mod ord_equality;
mod partial_impl;
mod same_type;
mod tuple_record;
mod type_shape;
use crate::structural_lint::languages::measure::{Code, Measurement};
use tree_sitter::Node;
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    match node.kind() {
        "source_file" => {
            ord_equality::inspect(node, source, output);
            derived_trait::unchecked_deserialize(node, source, output);
            derived_trait::open_checked_field(node, source, output);
        }
        "struct_item" => {
            derived_trait::secret_debug(node, source, output);
            same_type::positions(node, source, output);
        }
        "enum_variant" => same_type::positions(node, source, output),
        "enum_item" => implicit_order::inspect(node, source, output),
        "function_item" | "function_signature_item" => {
            tuple_record::record_signature(node, source, output);
            same_type::parameters(node, source, output);
        }
        "closure_expression" => tuple_record::record_signature(node, source, output),
        "type_item" => type_shape::alias(node, source, output),
        "generic_type" => type_shape::option(node, source, output),
        "impl_item" => partial_impl::inspect(node, source, output),
        "trait_item" => constant_default::inspect(node, source, output),
        _ => {}
    }
}
