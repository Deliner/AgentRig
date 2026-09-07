//! Syntax references, before filesystem/module resolution.
mod javascript;
mod python;
mod rust;
#[cfg(test)]
mod tests;

use super::Issue;
use crate::lint::languages;
use anyhow::{Context, Result};
use std::path::Path;
use tree_sitter::Node;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    RustPath { path: String, scope: Vec<String> },
    RustMacro(RustMacro),
    RustDerive { path: String, scope: Vec<String> },
    RustModule { path: Vec<String>, inline: bool },
    PythonModule(String),
    PythonFrom { module: String, names: Vec<String> },
    JavaScriptModule { path: String, loader: Loader },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustMacro {
    pub path: String,
    pub scope: Vec<String>,
    pub literal: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loader {
    Import,
    Require,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Reference {
    pub line: usize,
    pub target: Target,
}

#[derive(Default, Debug)]
pub struct References {
    pub items: Vec<Reference>,
    pub issues: Vec<Issue>,
    pub rust_items: Vec<RustItem>,
    pub rust_imports: Vec<RustImport>,
}

#[derive(Debug)]
pub struct RustItem {
    pub name: String,
    pub scope: Vec<String>,
}

#[derive(Debug)]
pub struct RustImport {
    pub name: String,
    pub path: String,
    pub scope: Vec<String>,
}

pub(super) struct Source<'a> {
    path: &'a Path,
    text: &'a str,
    output: References,
}

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

impl Source<'_> {
    fn loader_reference(&mut self, node: Node<'_>) {
        let indirect = !node
            .parent()
            .is_some_and(|parent| parent.child_by_field_name("function") == Some(node));
        if indirect {
            self.unsupported(
                node,
                "module loader used as a value needs alias/data-flow analysis",
            );
        }
    }

    fn text(&self, node: Node<'_>) -> &str {
        languages::text(node, self.text)
    }

    fn record(&mut self, node: Node<'_>, target: Target) {
        let target = match target {
            Target::RustPath { path, scope } => Target::RustPath {
                path: rust::prelude::path(self, node, rust::generic_path(self, node, path)),
                scope,
            },
            target => target,
        };
        let reference = Reference {
            line: node.start_position().row + 1,
            target,
        };
        let declaration = matches!(&reference.target, Target::RustModule { .. });
        let new = declaration || !self.output.items.contains(&reference);
        if new {
            self.output.items.push(reference);
        }
    }

    fn unsupported(&mut self, node: Node<'_>, reason: &str) {
        self.output.issues.push(Issue {
            path: self.path.into(),
            line: Some(node.start_position().row + 1),
            message: format!("dependency analysis incomplete: {reason}"),
        });
    }

    fn literal(&mut self, node: Node<'_>) -> Option<String> {
        let text = self.text(node);
        let interpolation = node
            .named_children(&mut node.walk())
            .any(|child| matches!(child.kind(), "interpolation" | "template_substitution"));
        let static_literal = node.kind() == "string" && !interpolation && !text.contains('\\');
        if static_literal {
            Some(
                node.named_children(&mut node.walk())
                    .filter(|child| matches!(child.kind(), "string_fragment" | "string_content"))
                    .map(|child| self.text(child))
                    .collect(),
            )
        } else {
            self.unsupported(node, "module path needs a static unescaped string literal");
            None
        }
    }
}
