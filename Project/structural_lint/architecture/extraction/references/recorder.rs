use super::{Reference, References, Target};
use crate::structural_lint::{architecture::Issue, languages};
use std::path::Path;
use tree_sitter::Node;

pub struct Source<'a> {
    pub path: &'a Path,
    pub text: languages::Code<'a>,
    pub output: References,
}

impl Source<'_> {
    pub fn loader_reference(&mut self, node: Node<'_>) {
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

    pub fn text(&self, node: Node<'_>) -> &str {
        languages::text(node, self.text)
    }

    pub fn record(&mut self, node: Node<'_>, target: Target) {
        self.add(Reference {
            line: node.start_position().row + 1,
            target,
            test: false,
        });
    }

    pub fn add(&mut self, reference: Reference) {
        let new = reference.target.repeatable() || !self.output.items.contains(&reference);
        if new {
            self.output.items.push(reference);
        }
    }

    pub fn unsupported(&mut self, node: Node<'_>, reason: &str) {
        self.output.issues.push(Issue {
            path: self.path.into(),
            line: Some(node.start_position().row + 1),
            message: format!("dependency analysis incomplete: {reason}"),
        });
    }

    pub fn literal(&mut self, node: Node<'_>) -> Option<String> {
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
