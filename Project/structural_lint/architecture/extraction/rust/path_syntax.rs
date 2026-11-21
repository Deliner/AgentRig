use crate::structural_lint::architecture::extraction::references::Source;
use tree_sitter::Node;

pub(super) fn compact(path: &str) -> String {
    path.split_whitespace()
        .collect::<String>()
        .replace("r#", "")
}

pub(super) fn string_literal(source: &Source<'_>, value: Node<'_>) -> Option<String> {
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

pub(super) fn scope(source: &Source<'_>, node: Node<'_>) -> Vec<String> {
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

pub(super) struct UseLeaf<'t> {
    pub(super) node: Node<'t>,
    pub(super) prefix: String,
    alias: Option<String>,
}

pub(super) fn use_leaves<'t>(
    source: &Source<'_>,
    node: Node<'t>,
    prefix: &str,
) -> Vec<UseLeaf<'t>> {
    let comment = node.is_extra();
    if comment {
        return Vec::new();
    }
    match node.kind() {
        "scoped_use_list" => {
            let path = node
                .child_by_field_name("path")
                .and_then(|n| path_text(source, n))
                .unwrap_or_default();
            let prefix = Prefix(prefix).join(&path);
            node.child_by_field_name("list")
                .map(|list| use_leaves(source, list, &prefix))
                .unwrap_or_default()
        }
        "use_list" => node
            .named_children(&mut node.walk())
            .flat_map(|child| use_leaves(source, child, prefix))
            .collect(),
        "use_as_clause" => aliased_leaf(source, node, prefix).into_iter().collect(),
        _ => vec![UseLeaf {
            node,
            prefix: prefix.into(),
            alias: None,
        }],
    }
}

fn aliased_leaf<'t>(source: &Source<'_>, clause: Node<'t>, prefix: &str) -> Option<UseLeaf<'t>> {
    let alias = clause
        .child_by_field_name("alias")
        .map(|n| compact(source.text(n)));
    let node = clause.child_by_field_name("path")?;
    Some(UseLeaf {
        node,
        prefix: prefix.into(),
        alias,
    })
}

pub(super) fn leaf_name(leaf: &UseLeaf<'_>, path: &str) -> String {
    let wildcard = leaf.node.kind() == "use_wildcard";
    leaf.alias.clone().unwrap_or_else(|| match wildcard {
        true => "*".into(),
        false => path.rsplit("::").next().unwrap_or("").into(),
    })
}

pub(super) fn leaf_path(source: &Source<'_>, node: Node<'_>, prefix: &str) -> Option<String> {
    let value = path_text(source, node)?;
    let grouped_self = matches!(value.as_str(), "self" | "*") && !prefix.is_empty();
    let path = if grouped_self {
        prefix.into()
    } else {
        Prefix(prefix).join(&value)
    };
    Some(self_path(source, node, path))
}

pub(super) fn names_import(source: &Source<'_>, node: Node<'_>, name: &str) -> bool {
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        let wildcard = node.kind() == "use_wildcard";
        let named = node.kind() == "identifier" && source.text(node) == name;
        let matches = wildcard || named;
        if matches {
            return true;
        }
        pending.extend(node.named_children(&mut node.walk()));
    }
    false
}

pub(super) fn self_path(source: &Source<'_>, node: Node<'_>, path: String) -> String {
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
                .map(|name| Prefix(&name).join(suffix))
                .unwrap_or(path);
        }
        let unrelated_item = matches!(owner.kind(), "mod_item" | "struct_item" | "enum_item")
            || local_function(owner);
        if unrelated_item {
            break;
        }
        parent = owner.parent();
    }
    path
}

pub(super) fn local_function(node: Node<'_>) -> bool {
    node.kind() == "function_item" && node.parent().is_some_and(|owner| owner.kind() == "block")
}

pub(super) fn path_text(source: &Source<'_>, node: Node<'_>) -> Option<String> {
    match node.kind() {
        "identifier" | "type_identifier" | "crate" | "self" | "super" => {
            Some(compact(source.text(node)))
        }
        "scoped_identifier" | "scoped_type_identifier" => {
            let name = path_text(source, node.child_by_field_name("name")?)?;
            match node.child_by_field_name("path") {
                Some(path) => Some(Prefix(&path_text(source, path)?).join(&name)),
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

#[derive(Clone, Copy)]
pub(super) struct Prefix<'a>(pub(super) &'a str);

impl Prefix<'_> {
    pub(super) fn join(self, path: &str) -> String {
        let Self(prefix) = self;
        let empty = prefix.is_empty();
        if empty {
            path.into()
        } else {
            format!("{prefix}::{path}")
        }
    }
}

pub(super) fn module_item(node: Node<'_>) -> bool {
    node.parent().is_some_and(|parent| {
        let inline_module = parent.kind() == "declaration_list"
            && parent
                .parent()
                .is_some_and(|owner| owner.kind() == "mod_item");
        parent.kind() == "source_file" || inline_module
    })
}
