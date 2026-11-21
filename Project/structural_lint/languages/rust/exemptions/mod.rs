mod async_code;
mod attributes;
mod cfg_flags;
mod debug_code;
mod foreign_trait;
mod path_names;
mod test_code;

use crate::structural_lint::languages::neutral_metrics::{Code, text};
pub(super) use async_code::{in_async_context, scope_boundary};
pub(super) use attributes::{attribute_path, attributed, derives, listed};
pub(super) use cfg_flags::cfg_excludes;
pub(super) use debug_code::in_debug_only_context;
pub(super) use foreign_trait::{extern_member, foreign_signature};
pub(super) use path_names::{last_segment, plain_name, type_name};
pub(crate) use test_code::in_test_context;
pub(super) use test_code::test_marker;
use tree_sitter::Node;

// DECISION: D224
pub(crate) const ENTRY_FILES: [&str; 3] = ["mod.rs", "lib.rs", "main.rs"];

pub(super) fn trait_impl_member(item: Node<'_>) -> bool {
    item.parent()
        .and_then(|list| list.parent())
        .is_some_and(|owner| {
            owner.kind() == "impl_item" && owner.child_by_field_name("trait").is_some()
        })
}
pub(super) fn zero_literal(node: Node<'_>, source: Code<'_>) -> bool {
    let value = text(node, source);
    let digits = match node.kind() {
        "integer_literal" => {
            let radix = ["0x", "0o", "0b"]
                .into_iter()
                .find_map(|prefix| value.strip_prefix(prefix));
            radix.unwrap_or(value).split(['u', 'i']).next()
        }
        "float_literal" => value.split(['e', 'E', 'f']).next(),
        _ => None,
    };
    digits.is_some_and(|digits| digits.chars().all(|digit| matches!(digit, '0' | '_' | '.')))
}
