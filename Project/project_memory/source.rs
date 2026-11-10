// DECISION: D002
use crate::shared::fs;
use anyhow::{Result, ensure};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use tree_sitter::Node;

pub struct Source {
    pub comments: HashSet<String>,
    pub functions: HashMap<String, Vec<HashSet<String>>>,
}
pub fn inspect(path: &Path) -> Result<Option<Source>> {
    let bytes = fs::read(path)?;
    let Ok(source) = String::from_utf8(bytes) else {
        return Ok(None);
    };
    let shebang = source.lines().next().unwrap_or("");
    let shell = path.extension().is_none() && shebang.starts_with("#!") && shebang.contains("sh");
    let parse_path = if shell { Path::new("source.sh") } else { path };
    let Some(tree) = crate::shared::syntax::parse(parse_path, &source)? else {
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
        result.visit(node, &source);
        let mut cursor = node.walk();
        nodes.extend(node.named_children(&mut cursor));
    }
    Ok(Some(result))
}
impl Source {
    fn visit(&mut self, node: Node<'_>, source: &str) {
        match node.kind() {
            "comment" | "line_comment" => {
                self.comments.extend(
                    source
                        .get(node.byte_range())
                        .map(|comment| comment.trim().to_owned()),
                );
            }
            "function_definition" | "function_item" | "function_signature_item" => {
                if let Some(name) = node.child_by_field_name("name") {
                    self.functions
                        .entry(qualified_name(node, source, name))
                        .or_default()
                        .push(preceding_markers(node, source));
                }
            }
            _ => {}
        }
    }

    pub fn marked_function(&self, name: &str, id: &str) -> bool {
        self.functions.get(name).is_some_and(|functions| {
            functions.len() == 1
                && (functions[0].contains(&format!("# INVARIANT: {id}"))
                    || functions[0].contains(&format!("// INVARIANT: {id}")))
        })
    }
    pub fn references(&self) -> impl Iterator<Item = &str> {
        self.comments
            .iter()
            .filter_map(|comment| crate::shared::syntax::reference(comment))
    }
    pub fn marker(&self, kind: &str, id: &str) -> bool {
        self.comments.contains(&format!("# {kind}: {id}"))
            || self.comments.contains(&format!("// {kind}: {id}"))
    }
}

fn preceding_markers(mut node: Node<'_>, source: &str) -> HashSet<String> {
    let mut markers = HashSet::new();
    loop {
        let original = node;
        'siblings: while let Some(previous) = node.prev_named_sibling() {
            let adjacent = source
                .get(previous.end_byte()..node.start_byte())
                .is_some_and(|gap| gap.trim().is_empty());
            let separated = !adjacent;
            if separated {
                break 'siblings;
            }
            match previous.kind() {
                "comment" | "line_comment" => {
                    markers.extend(
                        source
                            .get(previous.byte_range())
                            .map(|comment| comment.trim().to_owned()),
                    );
                }
                "attribute_item" => {}
                _ => break 'siblings,
            }
            node = previous;
        }
        let Some(parent) = original
            .parent()
            .filter(|p| p.kind() == "decorated_definition")
        else {
            break;
        };
        node = parent;
    }
    markers
}

fn qualified_name(node: Node<'_>, source: &str, name: Node<'_>) -> String {
    let mut parts: Vec<&str> = source.get(name.byte_range()).into_iter().collect();
    let mut parent = node.parent();
    while let Some(node) = parent {
        let scope = matches!(
            node.kind(),
            "mod_item" | "class_definition" | "function_definition" | "function_item"
        );
        if let Some(name) = node.child_by_field_name("name").filter(|_| scope) {
            parts.extend(source.get(name.byte_range()));
        }
        parent = node.parent();
    }
    parts.reverse();
    parts.join("::")
}
