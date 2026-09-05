use anyhow::{Result, ensure};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};
use tree_sitter::Node;

pub struct Source {
    pub comments: HashSet<String>,
    pub functions: HashMap<String, HashSet<String>>,
}
pub fn inspect(path: &Path) -> Result<Option<Source>> {
    let source = fs::read_to_string(path)?;
    let Some(tree) = crate::lint::languages::parse(path, &source)? else {
        return Ok(None);
    };
    ensure!(
        !tree.root_node().has_error(),
        "cannot inspect markers in malformed source {}",
        path.display()
    );
    let mut result = Source {
        comments: HashSet::new(),
        functions: HashMap::new(),
    };
    let mut nodes = vec![tree.root_node()];
    while let Some(node) = nodes.pop() {
        match node.kind() {
            "comment" | "line_comment" => {
                result
                    .comments
                    .insert(source[node.byte_range()].trim().to_owned());
            }
            "function_definition" | "function_item" | "function_signature_item" => {
                if let Some(name) = node.child_by_field_name("name") {
                    result
                        .functions
                        .entry(source[name.byte_range()].to_owned())
                        .or_default()
                        .extend(preceding_markers(node, &source));
                }
            }
            _ => {}
        }
        let mut cursor = node.walk();
        nodes.extend(node.named_children(&mut cursor));
    }
    Ok(Some(result))
}
impl Source {
    pub fn marked_function(&self, name: &str, id: &str) -> bool {
        self.functions.get(name).is_some_and(|markers| {
            markers.contains(&format!("# INVARIANT: {id}"))
                || markers.contains(&format!("// INVARIANT: {id}"))
        })
    }
    pub fn marker(&self, kind: &str, id: &str) -> bool {
        self.comments.contains(&format!("# {kind}: {id}"))
            || self.comments.contains(&format!("// {kind}: {id}"))
    }
}

fn preceding_markers(mut node: Node<'_>, source: &str) -> HashSet<String> {
    if let Some(parent) = node
        .parent()
        .filter(|parent| parent.kind() == "decorated_definition")
    {
        node = parent;
    }
    let mut markers = HashSet::new();
    while let Some(previous) = node.prev_named_sibling() {
        let adjacent = source[previous.end_byte()..node.start_byte()]
            .trim()
            .is_empty();
        if !adjacent {
            break;
        }
        match previous.kind() {
            "comment" | "line_comment" => {
                markers.insert(source[previous.byte_range()].trim().to_owned());
            }
            "attribute_item" => {}
            _ => break,
        }
        node = previous;
    }
    markers
}
