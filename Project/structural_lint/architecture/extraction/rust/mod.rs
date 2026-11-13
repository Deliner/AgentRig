mod attributes;
mod calls;
mod conditions;
mod macros;
pub(super) mod prelude;

use super::references::{Reference, RustImport, RustItem, RustMacro, Source, Target};
use crate::structural_lint::languages;
use tree_sitter::Node;

fn record(source: &mut Source<'_>, node: Node<'_>, target: Target) {
    let target = match target {
        Target::RustPath { path, scope } => Target::RustPath {
            path: prelude::path(source, node, generic_path(source, node, path)),
            scope,
        },
        target @ (Target::RustMacro(_)
        | Target::RustDerive { .. }
        | Target::RustModule { .. }
        | Target::RustInclude { .. }
        | Target::PythonModule(_)
        | Target::PythonFrom { .. }) => target,
    };
    // DECISION: D146
    let test = languages::in_test_context(node, source.text);
    source.add(Reference {
        line: node.start_position().row + 1,
        target,
        test,
    });
}

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

struct UseLeaf<'t> {
    node: Node<'t>,
    prefix: String,
    alias: Option<String>,
}

fn use_leaves<'t>(source: &Source<'_>, node: Node<'t>, prefix: &str) -> Vec<UseLeaf<'t>> {
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
        "use_as_clause" => {
            let alias = node
                .child_by_field_name("alias")
                .map(|n| compact(source.text(n)));
            let path = node.child_by_field_name("path");
            path.map(|node| UseLeaf {
                node,
                prefix: prefix.into(),
                alias,
            })
            .into_iter()
            .collect()
        }
        _ => vec![UseLeaf {
            node,
            prefix: prefix.into(),
            alias: None,
        }],
    }
}

fn leaf_name(leaf: &UseLeaf<'_>, path: &str) -> String {
    let wildcard = leaf.node.kind() == "use_wildcard";
    leaf.alias.clone().unwrap_or_else(|| match wildcard {
        true => "*".into(),
        false => path.rsplit("::").next().unwrap_or("").into(),
    })
}

fn import_leaf(source: &mut Source<'_>, leaf: UseLeaf<'_>) {
    let Some(path) = use_leaf(source, leaf.node, &leaf.prefix) else {
        return;
    };
    let name = leaf_name(&leaf, &path);
    let node = leaf.node;
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

fn leaf_path(source: &Source<'_>, node: Node<'_>, prefix: &str) -> Option<String> {
    let value = path_text(source, node)?;
    let grouped_self = matches!(value.as_str(), "self" | "*") && !prefix.is_empty();
    let path = if grouped_self {
        prefix.into()
    } else {
        Prefix(prefix).join(&value)
    };
    Some(self_path(source, node, path))
}

fn use_leaf(source: &mut Source<'_>, node: Node<'_>, prefix: &str) -> Option<String> {
    let Some(path) = leaf_path(source, node, prefix) else {
        source.unsupported(node, "Rust qualified path requires type resolution");
        return None;
    };
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

pub(super) fn generic_path(source: &Source<'_>, node: Node<'_>, path: String) -> String {
    let Some((name, suffix)) = path.split_once("::") else {
        return path;
    };
    let mut parent = node.parent();
    while let Some(owner) = parent {
        if let Some(binding) = lexical_binding(source, owner, name) {
            return match binding {
                Binding::Path(bound) => Prefix(&bound).join(suffix),
                Binding::Opaque => path,
            };
        }
        let separate_scope = owner.kind() == "mod_item" || local_function(owner);
        if separate_scope {
            break;
        }
        parent = owner.parent();
    }
    path
}

fn local_function(node: Node<'_>) -> bool {
    node.kind() == "function_item" && node.parent().is_some_and(|owner| owner.kind() == "block")
}

// DECISION: D136
enum Binding {
    Path(String),
    Opaque,
}

fn lexical_binding(source: &Source<'_>, owner: Node<'_>, name: &str) -> Option<Binding> {
    let block = owner.kind() == "block";
    let bound = if block {
        let binding = owner
            .named_children(&mut owner.walk())
            .find(|item| prelude::names(source, *item, name))?;
        prelude::imported(source, binding, name)
    } else {
        let parameters = owner.child_by_field_name("type_parameters")?;
        let parameter = parameters
            .named_children(&mut parameters.walk())
            .find(|item| {
                item.child_by_field_name("name")
                    .is_some_and(|binding| source.text(binding) == name)
            })?;
        single_bound(source, parameter)
    };
    Some(bound.map_or(Binding::Opaque, Binding::Path))
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
struct Prefix<'a>(&'a str);

impl Prefix<'_> {
    fn join(self, path: &str) -> String {
        let Self(prefix) = self;
        let empty = prefix.is_empty();
        if empty {
            path.into()
        } else {
            format!("{prefix}::{path}")
        }
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
        let inline_module = parent.kind() == "declaration_list"
            && parent
                .parent()
                .is_some_and(|owner| owner.kind() == "mod_item");
        parent.kind() == "source_file" || inline_module
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

// DECISION: D136
fn attribute(source: &mut Source<'_>, node: Node<'_>) {
    let path = node
        .named_children(&mut node.walk())
        .find(|child| child.kind() == "attribute")
        .and_then(|attribute| attribute_path(source, attribute))
        .unwrap_or_default();
    let builtin = matches!(
        path.as_str(),
        "allow"
            | "warn"
            | "deny"
            | "forbid"
            | "expect"
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
            | "no_mangle"
            | "export_name"
    );
    if builtin {
        return;
    }
    let call = RustMacro {
        path,
        scope: scope(source, node),
        literal: None,
    };
    record(source, node, Target::RustMacro(call));
}

fn attribute_path(source: &Source<'_>, attribute: Node<'_>) -> Option<String> {
    let path = compact(source.text(attribute.named_child(0)?));
    let unsafe_wrapper = path == "unsafe";
    if unsafe_wrapper {
        let arguments = attribute.child_by_field_name("arguments")?;
        let inner = arguments.named_child(0)?;
        return Some(compact(source.text(inner)));
    }
    Some(path)
}
