use crate::structural_lint::languages::neutral_metrics::{Code, Project, text};
use tree_sitter::Node;

// DECISION: D229
const PRELUDE_TRAITS: [&str; 34] = [
    "AsMut",
    "AsRef",
    "AsyncFn",
    "AsyncFnMut",
    "AsyncFnOnce",
    "Clone",
    "Copy",
    "Default",
    "DoubleEndedIterator",
    "Drop",
    "Eq",
    "ExactSizeIterator",
    "Extend",
    "Fn",
    "FnMut",
    "FnOnce",
    "From",
    "FromIterator",
    "Future",
    "Into",
    "IntoFuture",
    "IntoIterator",
    "Iterator",
    "Ord",
    "PartialEq",
    "PartialOrd",
    "Send",
    "Sized",
    "Sync",
    "ToOwned",
    "ToString",
    "TryFrom",
    "TryInto",
    "Unpin",
];
// DECISION: D229
const TYPE_ITEMS: [&str; 6] = [
    "mod_item",
    "trait_item",
    "struct_item",
    "enum_item",
    "union_item",
    "type_item",
];
// DECISION: D229
const IMPORT_DEPTH: usize = 8;

// DECISION: D229
pub fn extern_member(item: Node<'_>) -> bool {
    owner(item).is_some_and(|owner| owner.kind() == "foreign_mod_item")
}
// DECISION: D229
pub fn foreign_signature(item: Node<'_>, source: Code<'_>, project: &dyn Project) -> bool {
    let external = extern_member(item);
    if external {
        return true;
    }
    let Some(implementation) = owner(item).filter(|owner| owner.kind() == "impl_item") else {
        return false;
    };
    let written = implementation
        .child_by_field_name("trait")
        .and_then(|path| trait_path(path, source));
    written
        .is_some_and(|path| Lookup { source, project }.foreign(implementation, &path, IMPORT_DEPTH))
}
fn owner(item: Node<'_>) -> Option<Node<'_>> {
    item.parent().and_then(|list| list.parent())
}
fn trait_path(node: Node<'_>, source: Code<'_>) -> Option<String> {
    match node.kind() {
        "type_identifier" | "scoped_type_identifier" => Some(compact(text(node, source))),
        "generic_type" => trait_path(node.child_by_field_name("type")?, source),
        _ => None,
    }
}
fn compact(path: &str) -> String {
    path.split_whitespace().collect()
}
fn first_segment(path: &str) -> &str {
    path.split_once("::").map_or(path, |(first, _)| first)
}
fn bound_name(path: &str) -> &str {
    path.rsplit_once("::").map_or(path, |(_, last)| last)
}
#[derive(Clone, Copy)]
struct Prefix<'p>(&'p str);
impl Prefix<'_> {
    fn join(self, path: &str) -> String {
        let Self(prefix) = self;
        match (prefix.is_empty(), path.is_empty()) {
            (true, _) => path.to_owned(),
            (false, true) => prefix.to_owned(),
            (false, false) => format!("{prefix}::{path}"),
        }
    }
}

// DECISION: D229
enum Binding<'t> {
    Own,
    Free,
    Import { path: String, declaration: Node<'t> },
}
fn preferred<'t>(found: impl Iterator<Item = Binding<'t>>) -> Option<Binding<'t>> {
    let mut own = None;
    for binding in found {
        match binding {
            Binding::Own => own = Some(Binding::Own),
            explicit @ (Binding::Free | Binding::Import { .. }) => return Some(explicit),
        }
    }
    own
}
#[derive(Clone, Copy)]
struct Lookup<'s> {
    source: Code<'s>,
    project: &'s dyn Project,
}
impl Lookup<'_> {
    fn foreign(self, at: Node<'_>, path: &str, depth: usize) -> bool {
        if let Some(global) = path.strip_prefix("::") {
            return self.project.foreign_crate(first_segment(global));
        }
        let first = first_segment(path);
        match self.binding(at, first) {
            Binding::Own => false,
            Binding::Import { path, declaration } => {
                depth > 0 && self.foreign(declaration, &path, depth - 1)
            }
            Binding::Free => {
                let prelude = first == path && PRELUDE_TRAITS.contains(&first);
                self.project.foreign_crate(first) || prelude
            }
        }
    }
    fn binding<'t>(self, at: Node<'t>, name: &str) -> Binding<'t> {
        let relative = matches!(name, "crate" | "self" | "super" | "Self");
        if relative {
            return Binding::Own;
        }
        let mut inner = at;
        while let Some(scope) = inner.parent() {
            let module = module_scope(scope);
            let searched = module || scope.kind() == "block";
            let found = searched.then(|| self.beside(inner, name)).flatten();
            if let Some(binding) = found {
                return binding;
            }
            if module {
                return Binding::Free;
            }
            inner = scope;
        }
        Binding::Free
    }
    fn beside<'t>(self, inner: Node<'t>, name: &str) -> Option<Binding<'t>> {
        let scope = inner.parent()?;
        preferred(
            scope
                .named_children(&mut scope.walk())
                .filter(|item| *item != inner)
                .filter_map(|item| self.item_binding(item, name)),
        )
    }
    fn item_binding<'t>(self, item: Node<'t>, name: &str) -> Option<Binding<'t>> {
        let kind = item.kind();
        let imports = kind == "use_declaration";
        if imports {
            let search = Search {
                name,
                source: self.source,
                declaration: item,
            };
            return search.imported(item.child_by_field_name("argument")?, Prefix(""));
        }
        let named = TYPE_ITEMS.contains(&kind)
            && item
                .child_by_field_name("name")
                .is_some_and(|binding| text(binding, self.source) == name);
        named.then_some(Binding::Own)
    }
}
fn module_scope(node: Node<'_>) -> bool {
    let inline_module = node.kind() == "declaration_list"
        && node
            .parent()
            .is_some_and(|owner| owner.kind() == "mod_item");
    inline_module || node.kind() == "source_file"
}

struct Search<'s, 't> {
    name: &'s str,
    source: Code<'s>,
    declaration: Node<'t>,
}
impl<'t> Search<'_, 't> {
    fn imported(&self, tree: Node<'_>, prefix: Prefix<'_>) -> Option<Binding<'t>> {
        match tree.kind() {
            "scoped_use_list" => {
                let path = tree.child_by_field_name("path");
                let written = path.map(|path| compact(text(path, self.source)));
                let nested = prefix.join(written.as_deref().unwrap_or_default());
                self.imported(tree.child_by_field_name("list")?, Prefix(&nested))
            }
            "use_list" => preferred(
                tree.named_children(&mut tree.walk())
                    .filter_map(|item| self.imported(item, prefix)),
            ),
            "use_as_clause" => {
                let alias = tree.child_by_field_name("alias")?;
                let path = tree.child_by_field_name("path")?;
                let aliased = text(alias, self.source) == self.name;
                aliased.then(|| self.import(prefix.join(&compact(text(path, self.source)))))
            }
            "use_wildcard" => Some(Binding::Own),
            _ => {
                let written = compact(text(tree, self.source));
                let path = match written.as_str() {
                    "self" => prefix.join(""),
                    _ => prefix.join(&written),
                };
                let bound = bound_name(&path) == self.name;
                bound.then(|| self.import(path))
            }
        }
    }
    fn import(&self, path: String) -> Binding<'t> {
        Binding::Import {
            path,
            declaration: self.declaration,
        }
    }
}
