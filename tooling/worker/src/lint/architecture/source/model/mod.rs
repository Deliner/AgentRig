//! Shared syntax reference records and extraction context for language handlers.
use crate::lint::{architecture::Issue, languages};
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
    pub(super) path: &'a Path,
    pub(super) text: &'a str,
    pub(super) output: References,
}

impl Source<'_> {
    pub(super) fn loader_reference(&mut self, node: Node<'_>) {
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

    pub(super) fn text(&self, node: Node<'_>) -> &str {
        languages::text(node, self.text)
    }

    pub(super) fn record(&mut self, node: Node<'_>, target: Target) {
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

    pub(super) fn unsupported(&mut self, node: Node<'_>, reason: &str) {
        self.output.issues.push(Issue {
            path: self.path.into(),
            line: Some(node.start_position().row + 1),
            message: format!("dependency analysis incomplete: {reason}"),
        });
    }

    pub(super) fn literal(&mut self, node: Node<'_>) -> Option<String> {
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
