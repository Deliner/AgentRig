use super::neutral_metrics::{Code, Imported, Project, PythonImport, Source, descendants, text};
use crate::shared::source_parsers;
use tree_sitter::Node;

// DECISION: D229
const CHAIN_DEPTH: usize = 8;

// DECISION: D229
pub fn foreign_override(method: Node<'_>, source: Code<'_>, project: &dyn Project) -> bool {
    let Some(decorated) = method
        .parent()
        .filter(|parent| parent.kind() == "decorated_definition")
    else {
        return false;
    };
    let Some(class) = owning_class(decorated) else {
        return false;
    };
    let Some(name) = method.child_by_field_name("name") else {
        return false;
    };
    let file = File {
        module: module_root(class),
        code: source,
    };
    let marked = decorated
        .named_children(&mut decorated.walk())
        .filter(|child| child.kind() == "decorator")
        .any(|decorator| file.marks_override(decorator));
    let chain = Chain {
        project,
        method: text(name, source),
    };
    marked && chain.bases_foreign(file, class, CHAIN_DEPTH)
}
fn owning_class(decorated: Node<'_>) -> Option<Node<'_>> {
    let class = decorated
        .parent()?
        .parent()
        .filter(|class| class.kind() == "class_definition")?;
    let holder = class.parent()?;
    let wrapped = holder.kind() == "decorated_definition";
    let outer = if wrapped { holder.parent()? } else { holder };
    (outer.kind() == "module").then_some(class)
}
fn module_root(node: Node<'_>) -> Node<'_> {
    let mut root = node;
    while let Some(parent) = root.parent() {
        root = parent;
    }
    root
}
fn declares(class: Node<'_>, method: &str, code: Code<'_>) -> bool {
    let Some(body) = class.child_by_field_name("body") else {
        return false;
    };
    body.named_children(&mut body.walk()).any(|item| {
        let decorated = item.kind() == "decorated_definition";
        let definition = if decorated {
            item.child_by_field_name("definition")
        } else {
            Some(item)
        };
        definition
            .filter(|definition| definition.kind() == "function_definition")
            .and_then(|function| function.child_by_field_name("name"))
            .is_some_and(|name| text(name, code) == method)
    })
}
fn first_segment(path: &str) -> &str {
    path.split_once('.').map_or(path, |(first, _)| first)
}
// DECISION: D229
fn module_nodes(module: Node<'_>) -> Vec<Node<'_>> {
    let mut found = Vec::new();
    let mut pending = vec![module];
    while let Some(node) = pending.pop() {
        for child in node.named_children(&mut node.walk()) {
            let transparent = !matches!(
                child.kind(),
                "function_definition" | "class_definition" | "decorated_definition"
            );
            if transparent {
                pending.push(child);
            }
            found.push(child);
        }
    }
    found
}

// DECISION: D229
struct Named {
    module: String,
    name: String,
}
impl Named {
    fn submodule(&self) -> String {
        let relative_root = self.module.ends_with('.');
        let separator = if relative_root { "" } else { "." };
        format!(
            "{module}{separator}{name}",
            module = self.module,
            name = self.name
        )
    }
    fn import(&self) -> PythonImport<'_> {
        PythonImport {
            module: &self.module,
            name: &self.name,
        }
    }
}
enum Bound<'t> {
    Class(Node<'t>),
    Module(String),
    From(Named),
    Other,
}
struct Entry {
    path: String,
    alias: Option<String>,
}
#[derive(Clone, Copy)]
struct File<'t, 's> {
    module: Node<'t>,
    code: Code<'s>,
}
impl<'t> File<'t, '_> {
    fn marks_override(self, decorator: Node<'_>) -> bool {
        let Some(expression) = decorator.named_child(0) else {
            return false;
        };
        let written = text(expression, self.code);
        let named = written.rsplit('.').next() == Some("override");
        named
            && self
                .qualified(written)
                .is_some_and(|qualified| qualified == "typing.override")
    }
    fn qualified(self, written: &str) -> Option<String> {
        let (first, rest) = written.split_once('.').unwrap_or((written, ""));
        let prefix = match self.binding(first)? {
            Bound::Module(module) => module,
            Bound::From(named) => named.submodule(),
            Bound::Class(_) | Bound::Other => return None,
        };
        let bare = rest.is_empty();
        Some(if bare {
            prefix
        } else {
            format!("{prefix}.{rest}")
        })
    }
    fn binding(self, name: &str) -> Option<Bound<'t>> {
        let mut found = module_nodes(self.module)
            .into_iter()
            .filter_map(|node| self.bound(node, name));
        let first = found.next()?;
        Some(match found.next() {
            None => first,
            Some(_) => Bound::Other,
        })
    }
    fn star_import(self) -> bool {
        module_nodes(self.module)
            .iter()
            .any(|node| node.kind() == "wildcard_import")
    }
    fn bound(self, node: Node<'t>, name: &str) -> Option<Bound<'t>> {
        let named = |field| {
            node.child_by_field_name(field)
                .is_some_and(|written| text(written, self.code) == name)
        };
        match node.kind() {
            "import_statement" => self.imported_module(node, name),
            "import_from_statement" => self.imported_name(node, name),
            "decorated_definition" => self.bound(node.child_by_field_name("definition")?, name),
            "class_definition" => named("name").then_some(Bound::Class(node)),
            "function_definition" => named("name").then_some(Bound::Other),
            "assignment" => self.assigned(node, name).then_some(Bound::Other),
            _ => None,
        }
    }
    fn assigned(self, assignment: Node<'_>, name: &str) -> bool {
        assignment.child_by_field_name("left").is_some_and(|left| {
            descendants(left)
                .iter()
                .any(|part| part.kind() == "identifier" && text(*part, self.code) == name)
        })
    }
    fn entry(self, item: Node<'_>) -> Option<Entry> {
        match item.kind() {
            "aliased_import" => Some(Entry {
                path: text(item.child_by_field_name("name")?, self.code).to_owned(),
                alias: Some(text(item.child_by_field_name("alias")?, self.code).to_owned()),
            }),
            "dotted_name" => Some(Entry {
                path: text(item, self.code).to_owned(),
                alias: None,
            }),
            _ => None,
        }
    }
    fn imported_module(self, node: Node<'_>, name: &str) -> Option<Bound<'t>> {
        node.children_by_field_name("name", &mut node.walk())
            .filter_map(|item| self.entry(item))
            .find_map(|entry| {
                let module = match &entry.alias {
                    Some(_) => entry.path.clone(),
                    None => first_segment(&entry.path).to_owned(),
                };
                let bound = entry.alias.as_deref().unwrap_or(&module);
                (bound == name).then(|| Bound::Module(module.clone()))
            })
    }
    fn imported_name(self, node: Node<'_>, name: &str) -> Option<Bound<'t>> {
        let module = text(node.child_by_field_name("module_name")?, self.code);
        node.children_by_field_name("name", &mut node.walk())
            .filter_map(|item| self.entry(item))
            .find_map(|entry| {
                let bound = entry.alias.as_deref().unwrap_or(&entry.path);
                (bound == name).then(|| {
                    Bound::From(Named {
                        module: module.to_owned(),
                        name: entry.path.clone(),
                    })
                })
            })
    }
}

// DECISION: D229
#[derive(Clone, Copy)]
enum Unbound {
    Builtin,
    Untraced,
}
struct Chain<'p> {
    project: &'p dyn Project,
    method: &'p str,
}
impl Chain<'_> {
    fn bases_foreign(&self, file: File<'_, '_>, class: Node<'_>, depth: usize) -> bool {
        let Some(list) = class.child_by_field_name("superclasses") else {
            return true;
        };
        list.named_children(&mut list.walk())
            .filter(|base| !base.is_extra() && base.kind() != "keyword_argument")
            .all(|base| self.base_foreign(file, base, depth))
    }
    fn base_foreign(&self, file: File<'_, '_>, base: Node<'_>, depth: usize) -> bool {
        match base.kind() {
            "identifier" => {
                self.named_foreign(file, text(base, file.code), Unbound::Builtin, depth)
            }
            "attribute" => self.attribute_foreign(file, base, depth),
            "subscript" => base
                .child_by_field_name("value")
                .is_some_and(|value| self.base_foreign(file, value, depth)),
            _ => false,
        }
    }
    fn named_foreign(
        &self,
        file: File<'_, '_>,
        name: &str,
        unbound: Unbound,
        depth: usize,
    ) -> bool {
        match file.binding(name) {
            Some(Bound::Class(class)) => self.class_foreign(file, class, depth),
            Some(Bound::From(named)) => self.import_foreign(file, named.import(), depth),
            Some(Bound::Module(_) | Bound::Other) => false,
            None => match unbound {
                Unbound::Builtin => !file.star_import(),
                Unbound::Untraced => false,
            },
        }
    }
    fn attribute_foreign(&self, file: File<'_, '_>, attribute: Node<'_>, depth: usize) -> bool {
        let Some(qualified) = file.qualified(text(attribute, file.code)) else {
            return false;
        };
        let Some((module, name)) = qualified.rsplit_once('.') else {
            return false;
        };
        self.import_foreign(file, PythonImport { module, name }, depth)
    }
    fn class_foreign(&self, file: File<'_, '_>, class: Node<'_>, depth: usize) -> bool {
        let Some(deeper) = depth.checked_sub(1) else {
            return false;
        };
        let declared = declares(class, self.method, file.code);
        !declared && self.bases_foreign(file, class, deeper)
    }
    fn import_foreign(&self, file: File<'_, '_>, import: PythonImport<'_>, depth: usize) -> bool {
        let Some(deeper) = depth.checked_sub(1) else {
            return false;
        };
        match self.project.python_import(file.code.path(), import) {
            Imported::Outside => true,
            Imported::Unknown => false,
            Imported::Inside(sources) => sources
                .iter()
                .any(|source| self.module_foreign(source, import.name, deeper)),
        }
    }
    fn module_foreign(&self, source: &Source, name: &str, depth: usize) -> bool {
        let parsed = source_parsers::parse(&source.path, &source.text);
        let Ok(Some(tree)) = parsed else {
            return false;
        };
        let file = File {
            module: tree.root_node(),
            code: Code::new(&source.path, &source.text),
        };
        self.named_foreign(file, name, Unbound::Untraced, depth)
    }
}
