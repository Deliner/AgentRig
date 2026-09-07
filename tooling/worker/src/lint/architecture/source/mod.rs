//! Syntax references, before filesystem/module resolution.
mod javascript;
mod python;
mod rust;
#[cfg(test)]
mod tests;

pub(crate) mod model;
use crate::lint::languages;
use anyhow::{Context, Result};
use model::Source;
pub use model::{Loader, Reference, References, RustImport, RustItem, RustMacro, Target};
use std::path::Path;
use tree_sitter::Node;

pub fn extract(path: &Path, text: &str) -> Result<References> {
    let handler = languages::handler(path).context("unsupported dependency language")?;
    let tree = languages::parse(path, text)?.context("unsupported dependency language")?;
    let mut source = Source {
        path,
        text,
        output: References::default(),
    };
    let malformed = tree.root_node().has_error();
    if malformed {
        source.unsupported(first_error(tree.root_node()), "malformed source syntax");
        return Ok(source.output);
    }
    let mut pending = vec![tree.root_node()];
    while let Some(node) = pending.pop() {
        let descend = match handler.name {
            "rust" => rust::inspect(&mut source, node),
            "python" => python::inspect(&mut source, node),
            _ => javascript::inspect(&mut source, node),
        };
        if descend {
            let mut cursor = node.walk();
            pending.extend(
                node.named_children(&mut cursor)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev(),
            );
        }
    }
    Ok(source.output)
}

fn first_error(root: Node<'_>) -> Node<'_> {
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        let invalid = node.is_error() || node.is_missing();
        if invalid {
            return node;
        }
        let mut cursor = node.walk();
        pending.extend(
            node.children(&mut cursor)
                .collect::<Vec<_>>()
                .into_iter()
                .rev(),
        );
    }
    root
}
