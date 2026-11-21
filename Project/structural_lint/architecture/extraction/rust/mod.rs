mod attributes;
mod calls;
mod conditions;
mod macros;
mod path_syntax;
mod prelude;
mod recording;
#[cfg(test)]
mod tests;

use super::references::{RustItem, Source, Target};
use path_syntax::{compact, module_item, scope, use_leaves};
pub(crate) use prelude::origin as prelude_origin;
use recording::{import_leaf, record, use_leaf};
use tree_sitter::Node;

pub(super) fn inspect(source: &mut Source<'_>, node: Node<'_>) -> bool {
    match node.kind() {
        "use_declaration" => {
            if let Some(argument) = node.child_by_field_name("argument") {
                for leaf in use_leaves(source, argument, "") {
                    import_leaf(source, leaf);
                }
            }
            return false;
        }
        "scoped_identifier" | "scoped_type_identifier" => {
            use_leaf(source, node, "");
            return node
                .child_by_field_name("path")
                .is_some_and(|n| matches!(n.kind(), "generic_type" | "bracketed_type"));
        }
        "mod_item" => module(source, node),
        "function_item" | "struct_item" | "enum_item" | "union_item" | "type_item"
        | "trait_item" | "const_item" | "static_item" => item(source, node),
        "attribute_item" | "inner_attribute_item" => {
            attributes::inspect(source, node);
            return false;
        }
        "macro_definition" => {
            macros::definition(source, node);
            return false;
        }
        "macro_invocation" => {
            macros::inspect(source, node);
            return false;
        }
        "call_expression" => calls::inspect(source, node),
        "visibility_modifier" => return false,
        _ => {}
    }
    true
}

fn module(source: &mut Source<'_>, node: Node<'_>) {
    let local = !module_item(node);
    if local {
        source.unsupported(node, "block-local modules require lexical scope resolution");
        return;
    }
    if let Some(name) = node.child_by_field_name("name") {
        let mut path = scope(source, node);
        path.push(compact(source.text(name)));
        let file = attributes::module_path(source, node);
        let visible = node
            .named_child(0)
            .is_some_and(|first| first.kind() == "visibility_modifier");
        record(
            source,
            node,
            Target::RustModule {
                path,
                inline: node.child_by_field_name("body").is_some(),
                private: !visible,
                file,
            },
        );
    }
}

fn item(source: &mut Source<'_>, node: Node<'_>) {
    let module_binding = module_item(node);
    let name = module_binding
        .then(|| node.child_by_field_name("name"))
        .flatten();
    if let Some(name) = name {
        source.output.rust_items.push(RustItem {
            name: compact(source.text(name)),
            scope: scope(source, node),
        });
    }
}
