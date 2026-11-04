use super::test_context::in_test_context;
use super::{Measurement, justified, text};
use crate::structural_lint::rules::kind::Kind;
use std::path::Path;
use tree_sitter::Node;

// DECISION: D096
const LOCKS: &[&str] = &["Mutex", "RwLock", "DashMap", "DashSet"];
const CELLS: &[&str] = &["RefCell", "Cell", "UnsafeCell"];
const ALIAS_DEPTH: usize = 3;

pub(super) fn inspect(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let exempt = justified(node, source) || in_test_context(node, source);
    if exempt {
        return;
    }
    let Some(measurement) = measure(node, source) else {
        return;
    };
    output.push(measurement);
}
pub(super) fn macro_items(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let exempt = !declares_items(node, source) || in_test_context(node, source);
    if exempt {
        return;
    }
    let mut cursor = node.walk();
    let Some(tree) = node
        .children(&mut cursor)
        .find(|child| child.kind() == "token_tree")
    else {
        return;
    };
    let Some(fragment) = items_source(tree, source) else {
        return;
    };
    let Ok(Some(parsed)) = crate::shared::syntax::parse(Path::new("items.rs"), &fragment) else {
        return;
    };
    let root = parsed.root_node();
    let broken = root.has_error();
    if broken {
        return;
    }
    let body = Body {
        source: &fragment,
        base: tree.start_position().row,
        above: justified(node, source),
    };
    let mut cursor = root.walk();
    for item in root
        .children(&mut cursor)
        .filter(|child| child.kind() == "static_item")
    {
        body.report(item, output);
    }
}
fn declares_items(node: Node<'_>, source: &str) -> bool {
    let Some(path) = node.child_by_field_name("macro") else {
        return false;
    };
    let name = text(path, source).rsplit("::").next().unwrap_or_default();
    matches!(name, "lazy_static" | "thread_local")
}
fn items_source(tree: Node<'_>, source: &str) -> Option<String> {
    let span = tree.end_byte().saturating_sub(tree.start_byte());
    let empty = span < 2;
    if empty {
        return None;
    }
    let mut fragment = source[tree.start_byte() + 1..tree.end_byte() - 1].to_string();
    let terminated = fragment.trim_end().ends_with(';');
    if terminated {
        return Some(fragment);
    }
    fragment.push(';');
    Some(fragment)
}
struct Body<'a> {
    source: &'a str,
    base: usize,
    above: bool,
}
impl Body<'_> {
    fn report(&self, item: Node<'_>, output: &mut Vec<Measurement>) {
        let first_row = item.start_position().row == 0;
        let exempt = justified(item, self.source) || (first_row && self.above);
        if exempt {
            return;
        }
        let Some(mut measurement) = measure(item, self.source) else {
            return;
        };
        measurement.line += self.base;
        output.push(measurement);
    }
}
fn measure(item: Node<'_>, source: &str) -> Option<Measurement> {
    let mutable = has_child(item, "mutable_specifier");
    if mutable {
        return None;
    }
    let annotation = item.child_by_field_name("type")?;
    let scope = Scope {
        root: root(item),
        source,
    };
    let mut names = Vec::new();
    scope.names(annotation, 0, &mut names);
    let locked = names.iter().any(|name| LOCKS.contains(name));
    let exposed = has_child(item, "visibility_modifier") && names.iter().any(|name| cell(name));
    let owned = !(locked || exposed);
    if owned {
        return None;
    }
    let symbol = item
        .child_by_field_name("name")
        .map(|name| text(name, source))
        .unwrap_or("static");
    Some(Measurement {
        kind: Kind::GlobalState,
        line: item.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    })
}
fn cell(name: &str) -> bool {
    LOCKS.contains(&name) || CELLS.contains(&name) || name.starts_with("Atomic")
}
fn has_child(item: Node<'_>, kind: &str) -> bool {
    let mut cursor = item.walk();
    item.children(&mut cursor).any(|child| child.kind() == kind)
}
fn root(node: Node<'_>) -> Node<'_> {
    let mut current = node;
    while let Some(parent) = current.parent() {
        current = parent;
    }
    current
}
struct Scope<'a> {
    root: Node<'a>,
    source: &'a str,
}
impl<'a> Scope<'a> {
    fn names(&self, annotation: Node<'a>, depth: usize, found: &mut Vec<&'a str>) {
        let mut pending = vec![annotation];
        while let Some(node) = pending.pop() {
            let mut cursor = node.walk();
            pending.extend(node.named_children(&mut cursor));
            let plain = node.kind() != "type_identifier";
            if plain {
                continue;
            }
            let name = text(node, self.source);
            found.push(name);
            let deep = depth >= ALIAS_DEPTH;
            if deep {
                continue;
            }
            let Some(target) = self.alias(name) else {
                continue;
            };
            self.names(target, depth + 1, found);
        }
    }
    fn alias(&self, name: &str) -> Option<Node<'a>> {
        let mut cursor = self.root.walk();
        self.root
            .children(&mut cursor)
            .find(|node| {
                node.kind() == "type_item"
                    && node
                        .child_by_field_name("name")
                        .is_some_and(|declared| text(declared, self.source) == name)
            })
            .and_then(|node| node.child_by_field_name("type"))
    }
}
