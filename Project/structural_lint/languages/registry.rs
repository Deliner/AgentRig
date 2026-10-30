// DECISION: D033
use super::{Measurement, python, rust};
use crate::structural_lint::rules::kind::Kind;
use std::path::Path;
use tree_sitter::Node;

pub struct Handler {
    pub name: &'static str,
    pub title: &'static str,
    pub extensions: &'static [&'static str],
    pub rules: &'static [Kind],
    pub inspect: Option<fn(Node<'_>, &str, &mut Vec<Measurement>)>,
}
pub const HANDLERS: &[Handler] = &[
    Handler {
        name: "rust",
        title: "Rust",
        extensions: &[".rs"],
        rules: &[
            Kind::NamedIfCondition,
            Kind::FunctionLines,
            Kind::ParameterCount,
            Kind::NestingDepth,
            Kind::BooleanParameter,
            Kind::BranchBudget,
            Kind::CommentReference,
            Kind::DirectoryArchitecture,
        ],
        inspect: Some(rust::inspect),
    },
    Handler {
        name: "python",
        title: "Python",
        extensions: &[".py", ".pyi"],
        rules: &[
            Kind::NamedIfCondition,
            Kind::FunctionLines,
            Kind::ParameterCount,
            Kind::NestingDepth,
            Kind::BooleanParameter,
            Kind::BranchBudget,
            Kind::CommentReference,
            Kind::DirectoryArchitecture,
        ],
        inspect: Some(python::inspect),
    },
];
pub fn handler(path: &Path) -> Option<&'static Handler> {
    let extension = format!(".{}", path.extension()?.to_str()?);
    HANDLERS
        .iter()
        .find(|handler| handler.extensions.contains(&extension.as_str()))
}
