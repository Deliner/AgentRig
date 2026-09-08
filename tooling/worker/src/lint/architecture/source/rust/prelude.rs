use super::{Source, compact};
use tree_sitter::Node;

pub fn path(source: &Source<'_>, node: Node<'_>, path: String) -> String {
    let (name, suffix) = path.split_once("::").unwrap_or((&path, ""));
    let origin = match name {
        "String" => "std::string::String",
        "Vec" => "std::vec::Vec",
        "Default" => "std::default::Default",
        "Option" => "std::option::Option",
        "Into" => "std::convert::Into",
        "str" | "bool" | "char" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8"
        | "u16" | "u32" | "u64" | "u128" | "usize" | "f32" | "f64" => {
            return primitive(source, node, path);
        }
        _ => return path,
    };
    let shadowed = shadowed(source, node, name);
    if shadowed {
        return path;
    }
    match suffix.is_empty() {
        true => format!("::{origin}"),
        false => format!("::{origin}::{suffix}"),
    }
}

fn primitive(source: &Source<'_>, node: Node<'_>, path: String) -> String {
    let name = path.split("::").next().unwrap_or(&path);
    let shadowed = shadowed(source, node, name);
    if shadowed {
        path
    } else {
        format!("::core::primitive::{path}")
    }
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

pub(super) fn names(source: &Source<'_>, item: Node<'_>, name: &str) -> bool {
    let imported = item.kind() == "use_declaration";
    if imported {
        return super::macros::names_import(source, item, name);
    }
    item.child_by_field_name("name")
        .is_some_and(|binding| compact(source.text(binding)) == name)
}
