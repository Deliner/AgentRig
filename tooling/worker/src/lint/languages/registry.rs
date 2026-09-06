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
        ],
        grammar: || tree_sitter_rust::LANGUAGE.into(),
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
        ],
        grammar: || tree_sitter_python::LANGUAGE.into(),
        inspect: Some(python::inspect),
    },
    Handler {
        name: "javascript",
        title: "JavaScript",
        extensions: &[".js", ".jsx", ".mjs", ".cjs"],
        rules: &[],
        grammar: || tree_sitter_javascript::LANGUAGE.into(),
        inspect: None,
    },
    Handler {
        name: "typescript",
        title: "TypeScript",
        extensions: &[".ts", ".mts", ".cts"],
        rules: &[],
        grammar: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        inspect: None,
    },
    Handler {
        name: "tsx",
        title: "TSX",
        extensions: &[".tsx"],
        rules: &[],
        grammar: || tree_sitter_typescript::LANGUAGE_TSX.into(),
        inspect: None,
    },
];
pub fn handler(path: &Path) -> Option<&'static Handler> {
    let extension = format!(".{}", path.extension()?.to_str()?);
    HANDLERS
        .iter()
        .find(|handler| handler.extensions.contains(&extension.as_str()))
}
