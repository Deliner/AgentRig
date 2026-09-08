mod attributes;
mod macros;
pub(super) mod prelude;

use super::model::{RustImport, RustItem, Source, Target};
use tree_sitter::Node;

fn record(source: &mut Source<'_>, node: Node<'_>, target: Target) {
    let target = match target {
        Target::RustPath { path, scope } => Target::RustPath {
            path: prelude::path(source, node, generic_path(source, node, path)),
            scope,
        },
        target => target,
    };
    source.record(node, target);
}

pub(super) fn inspect(source: &mut Source<'_>, node: Node<'_>) -> bool {
    match node.kind() {
        "use_declaration" => {
            if let Some(argument) = node.child_by_field_name("argument") {
                use_tree(source, argument, "");
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
        "macro_definition" => source.unsupported(
            node,
            "Rust macro definitions may generate dependencies; expansion is not analyzed",
        ),
        "macro_invocation" => {
            macros::inspect(source, node);
            return false;
        }
        _ => {}
    }
    true
}

fn compact(path: &str) -> String {
    path.split_whitespace()
        .collect::<String>()
        .replace("r#", "")
}

fn string_literal(source: &Source<'_>, value: Node<'_>) -> Option<String> {
    let string = matches!(value.kind(), "string_literal" | "raw_string_literal");
    if string {
        let text = source.text(value);
        let start = text.find('"')? + 1;
        let end = text.rfind('"')?;
        let value = text.get(start..end)?;
        return (!value.contains('\\')).then(|| value.to_owned());
    }
    None
}

fn scope(source: &Source<'_>, node: Node<'_>) -> Vec<String> {
    let mut scope = Vec::new();
    let mut parent = node.parent();
    while let Some(node) = parent {
        let module = node.kind() == "mod_item";
        let module_name = module.then(|| node.child_by_field_name("name")).flatten();
        if let Some(name) = module_name {
            scope.push(compact(source.text(name)));
        }
        parent = node.parent();
    }
    scope.reverse();
    scope
}

fn use_tree(source: &mut Source<'_>, node: Node<'_>, prefix: &str) {
    let comment = node.is_extra();
    if comment {
        return;
    }
    match node.kind() {
        "scoped_use_list" => {
            let path = node
                .child_by_field_name("path")
                .and_then(|n| path_text(source, n))
                .unwrap_or_default();
            let prefix = joined(prefix, &path);
            if let Some(list) = node.child_by_field_name("list") {
                use_tree(source, list, &prefix);
            }
        }
        "use_list" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                use_tree(source, child, prefix);
            }
        }
        "use_as_clause" => {
            let alias = node
                .child_by_field_name("alias")
                .map(|n| compact(source.text(n)));
            if let Some(path) = node.child_by_field_name("path") {
                import_leaf(source, path, prefix, alias);
            }
        }
        _ => import_leaf(source, node, prefix, None),
    }
}

fn import_leaf(source: &mut Source<'_>, node: Node<'_>, prefix: &str, alias: Option<String>) {
    let Some(path) = use_leaf(source, node, prefix) else {
        return;
    };
    let wildcard = node.kind() == "use_wildcard";
    let name = alias.unwrap_or_else(|| match wildcard {
        true => "*".into(),
        false => path.rsplit("::").next().unwrap_or("").into(),
    });
    let mut declaration = node;
    while let Some(parent) = declaration.parent() {
        let use_item = declaration.kind() == "use_declaration";
        if use_item {
            break;
        }
        declaration = parent;
    }
    let module_binding = module_item(declaration);
    if module_binding {
        source.output.rust_imports.push(RustImport {
            name,
            path,
            scope: scope(source, node),
        });
    }
}

fn use_leaf(source: &mut Source<'_>, node: Node<'_>, prefix: &str) -> Option<String> {
    let Some(value) = path_text(source, node) else {
        source.unsupported(node, "Rust qualified path requires type resolution");
        return None;
    };
    let grouped_self = matches!(value.as_str(), "self" | "*") && !prefix.is_empty();
    let path = if grouped_self {
        prefix.into()
    } else {
        joined(prefix, &value)
    };
    let path = self_path(source, node, path);
    record(
        source,
        node,
        Target::RustPath {
            path: path.clone(),
            scope: scope(source, node),
        },
    );
    Some(path)
}

fn self_path(source: &Source<'_>, node: Node<'_>, path: String) -> String {
    let Some(suffix) = path.strip_prefix("Self::") else {
        return path;
    };
    let mut parent = node.parent();
    while let Some(owner) = parent {
        let binding = match owner.kind() {
            "impl_item" => owner.child_by_field_name("type"),
            "trait_item" => owner.child_by_field_name("name"),
            _ => None,
        };
        if let Some(binding) = binding {
            return path_text(source, binding)
                .map(|name| joined(&name, suffix))
                .unwrap_or(path);
        }
        let unrelated_item = matches!(owner.kind(), "mod_item" | "struct_item" | "enum_item")
            || (owner.kind() == "function_item"
                && owner
                    .parent()
                    .is_some_and(|parent| parent.kind() == "block"));
        if unrelated_item {
            break;
        }
        parent = owner.parent();
    }
    path
}

pub(super) fn generic_path(source: &Source<'_>, node: Node<'_>, path: String) -> String {
    let Some((name, suffix)) = path.split_once("::") else {
        return path;
    };
    let mut parent = node.parent();
    while let Some(owner) = parent {
        let local_binding = owner.kind() == "block"
            && owner
                .named_children(&mut owner.walk())
                .any(|item| prelude::names(source, item, name));
        if local_binding {
            return path;
        }
        if let Some(parameters) = owner.child_by_field_name("type_parameters") {
            let parameter = parameters
                .named_children(&mut parameters.walk())
                .find(|item| {
                    item.child_by_field_name("name")
                        .is_some_and(|binding| source.text(binding) == name)
                });
            if let Some(parameter) = parameter {
                let bound = single_bound(source, parameter);
                return bound.map(|bound| joined(&bound, suffix)).unwrap_or(path);
            }
        }
        let separate_scope = owner.kind() == "mod_item"
            || (owner.kind() == "function_item"
                && owner.parent().is_some_and(|p| p.kind() == "block"));
        if separate_scope {
            break;
        }
        parent = owner.parent();
    }
    path
}

fn single_bound(source: &Source<'_>, parameter: Node<'_>) -> Option<String> {
    let bounds = parameter.child_by_field_name("bounds")?;
    let names: Vec<_> = bounds
        .named_children(&mut bounds.walk())
        .filter(|node| !node.is_extra() && node.kind() != "lifetime")
        .collect();
    let [bound] = names.as_slice() else {
        return None;
    };
    path_text(source, *bound)
}

fn path_text(source: &Source<'_>, node: Node<'_>) -> Option<String> {
    match node.kind() {
        "identifier" | "type_identifier" | "crate" | "self" | "super" => {
            Some(compact(source.text(node)))
        }
        "scoped_identifier" | "scoped_type_identifier" => {
            let name = path_text(source, node.child_by_field_name("name")?)?;
            match node.child_by_field_name("path") {
                Some(path) => Some(joined(&path_text(source, path)?, &name)),
                None => Some(format!("::{name}")),
            }
        }
        "generic_type" => path_text(source, node.child_by_field_name("type")?),
        "use_wildcard" => match node.named_child(0) {
            Some(path) => path_text(source, path),
            None => Some("*".into()),
        },
        _ => None,
    }
}

fn joined(prefix: &str, path: &str) -> String {
    let empty = prefix.is_empty();
    if empty {
        path.into()
    } else {
        format!("{prefix}::{path}")
    }
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
        record(
            source,
            node,
            Target::RustModule {
                path,
                inline: node.child_by_field_name("body").is_some(),
                file,
            },
        );
    }
}

fn module_item(node: Node<'_>) -> bool {
    node.parent().is_some_and(|parent| {
        parent.kind() == "source_file"
            || (parent.kind() == "declaration_list"
                && parent
                    .parent()
                    .is_some_and(|owner| owner.kind() == "mod_item"))
    })
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

fn attribute(source: &mut Source<'_>, node: Node<'_>) {
    let name = node
        .named_children(&mut node.walk())
        .find(|child| child.kind() == "attribute")
        .and_then(|attribute| attribute.named_child(0))
        .map(|name| source.text(name))
        .unwrap_or("");
    let path_override = name == "path";
    if path_override {
        source.unsupported(
            node,
            "Rust path attributes require explicit module resolution",
        );
    } else {
        let needs_expansion = !matches!(
            name,
            "allow"
                | "warn"
                | "deny"
                | "forbid"
                | "doc"
                | "inline"
                | "cold"
                | "must_use"
                | "deprecated"
                | "repr"
                | "non_exhaustive"
                | "test"
                | "ignore"
                | "should_panic"
                | "track_caller"
        );
        if needs_expansion {
            source.unsupported(
                node,
                "Rust attribute or conditional expansion is not analyzed",
            );
        }
    }
}
