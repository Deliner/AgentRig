use super::{Source, compact};
use tree_sitter::Node;

pub fn path(source: &Source<'_>, node: Node<'_>, path: String) -> String {
    let name = path.split("::").next().unwrap_or(&path);
    match origin(&path) {
        Some(origin) if !shadowed(source, node, name) => origin,
        _ => path,
    }
}

// DECISION: D136
pub fn origin(path: &str) -> Option<String> {
    let (name, suffix) = path.split_once("::").unwrap_or((path, ""));
    let origin = match name {
        "String" => "std::string::String".to_owned(),
        "Vec" => "std::vec::Vec".to_owned(),
        "Box" => "std::boxed::Box".to_owned(),
        "Default" => "std::default::Default".to_owned(),
        "Option" => "std::option::Option".to_owned(),
        "Into" => "std::convert::Into".to_owned(),
        "str" | "bool" | "char" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8"
        | "u16" | "u32" | "u64" | "u128" | "usize" | "f32" | "f64" => {
            format!("core::primitive::{name}")
        }
        _ => return None,
    };
    Some(match suffix.is_empty() {
        true => format!("::{origin}"),
        false => format!("::{origin}::{suffix}"),
    })
}

fn shadowed(source: &Source<'_>, node: Node<'_>, name: &str) -> bool {
    let mut parent = node.parent();
    while let Some(owner) = parent {
        let parameters = owner.child_by_field_name("type_parameters");
        if let Some(parameters) = parameters {
            let binding = parameters
                .named_children(&mut parameters.walk())
                .any(|item| names(source, item, name));
            if binding {
                return true;
            }
        }
        let scope = matches!(owner.kind(), "source_file" | "declaration_list" | "block");
        if scope {
            let binding = owner
                .named_children(&mut owner.walk())
                .any(|item| names(source, item, name));
            if binding {
                return true;
            }
        }
        let module = owner.kind() == "mod_item";
        if module {
            break;
        }
        parent = owner.parent();
    }
    false
}

// DECISION: D136
pub(super) fn imported(source: &Source<'_>, item: Node<'_>, name: &str) -> Option<String> {
    let declaration = item.kind() == "use_declaration";
    let argument = item
        .child_by_field_name("argument")
        .filter(|_| declaration)?;
    super::use_leaves(source, argument, "")
        .into_iter()
        .find_map(|leaf| {
            let path = super::leaf_path(source, leaf.node, &leaf.prefix)?;
            (super::leaf_name(&leaf, &path) == name).then_some(path)
        })
}

pub(super) fn names(source: &Source<'_>, item: Node<'_>, name: &str) -> bool {
    let imported = item.kind() == "use_declaration";
    if imported {
        return super::macros::names_import(source, item, name);
    }
    item.child_by_field_name("name")
        .is_some_and(|binding| compact(source.text(binding)) == name)
}
