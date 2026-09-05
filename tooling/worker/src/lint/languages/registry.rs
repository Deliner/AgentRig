use super::{Measurement, python, rust};
use crate::lint::rules::Kind;
use std::path::Path;
use tree_sitter::Node;

pub struct Handler {
    pub name: &'static str,
    pub title: &'static str,
    pub extensions: &'static [&'static str],
    pub rules: &'static [Kind],
    pub grammar: fn() -> tree_sitter::Language,
    pub inspect: fn(Node<'_>, &str, &mut Vec<Measurement>),
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
        ],
        grammar: || tree_sitter_rust::LANGUAGE.into(),
        inspect: rust::inspect,
    },
    Handler {
        name: "python",
        title: "Python",
        extensions: &[".py", ".pyi"],
        rules: &[
            Kind::NamedIfCondition,
            Kind::FunctionLines,
            Kind::ParameterCount,
        ],
        grammar: || tree_sitter_python::LANGUAGE.into(),
        inspect: python::inspect,
    },
];
pub fn handler(path: &Path) -> Option<&'static Handler> {
    let extension = format!(".{}", path.extension()?.to_str()?);
    HANDLERS
        .iter()
        .find(|handler| handler.extensions.contains(&extension.as_str()))
}
