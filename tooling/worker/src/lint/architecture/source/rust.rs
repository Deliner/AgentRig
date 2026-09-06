use super::{Source, Target};
use tree_sitter::Node;

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
        "attribute_item" => attribute(source, node),
        "macro_definition" => source.unsupported(
            node,
            "Rust macro definitions may generate dependencies; expansion is not analyzed",
        ),
        "macro_invocation" => include(source, node),
        _ => {}
    }
    true
}

fn compact(path: &str) -> String {
    path.split_whitespace()
        .collect::<String>()
        .replace("r#", "")
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
            if let Some(path) = node.child_by_field_name("path") {
                use_tree(source, path, prefix);
            }
        }
        _ => use_leaf(source, node, prefix),
    }
}

fn use_leaf(source: &mut Source<'_>, node: Node<'_>, prefix: &str) {
    let Some(value) = path_text(source, node) else {
        source.unsupported(node, "Rust qualified path requires type resolution");
        return;
    };
    let grouped_self = matches!(value.as_str(), "self" | "*") && !prefix.is_empty();
    let path = if grouped_self {
        prefix.into()
    } else {
        joined(prefix, &value)
    };
    source.record(
        node,
        Target::RustPath {
            path,
            scope: scope(source, node),
        },
    );
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
    if let Some(name) = node.child_by_field_name("name") {
        let mut path = scope(source, node);
        path.push(compact(source.text(name)));
        source.record(
            node,
            Target::RustModule {
                path,
                inline: node.child_by_field_name("body").is_some(),
            },
        );
    }
}

fn attribute(source: &mut Source<'_>, node: Node<'_>) {
    let path_override = compact(source.text(node)).contains("path=");
    if path_override {
        source.unsupported(
            node,
            "Rust path attributes require explicit module resolution",
        );
    }
}

fn include(source: &mut Source<'_>, node: Node<'_>) {
    let inclusion = node
        .child_by_field_name("macro")
        .is_some_and(|name| source.text(name) == "include");
    if inclusion {
        source.unsupported(node, "Rust include! source expansion is not analyzed");
    }
}
