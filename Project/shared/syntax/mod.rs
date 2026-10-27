use anyhow::{Context, Result};
use std::path::Path;
use tree_sitter::{Language, Parser, Tree};

pub fn parse(path: &Path, source: &str) -> Result<Option<Tree>> {
    let language: Language = match path.extension().and_then(|extension| extension.to_str()) {
        Some("rs") => tree_sitter_rust::LANGUAGE.into(),
        Some("py" | "pyi") => tree_sitter_python::LANGUAGE.into(),
        Some("sh") => tree_sitter_bash::LANGUAGE.into(),
        _ => return Ok(None),
    };
    let mut parser = Parser::new();
    parser.set_language(&language)?;
    Ok(Some(
        parser
            .parse(source, None)
            .context("parser returned no tree")?,
    ))
}
// DECISION: D037
pub fn reference(comment: &str) -> Option<&str> {
    let body = comment.trim_end();
    let marked = body.strip_prefix("//").or_else(|| body.strip_prefix('#'))?;
    let (label, id) = marked.strip_prefix(' ')?.split_once(": ")?;
    let prefix = match label {
        "DECISION" => "D",
        "INVARIANT" => "I",
        _ => return None,
    };
    let digits = id.strip_prefix(prefix)?;
    let numbered = digits.len() == 3 && digits.bytes().all(|byte| byte.is_ascii_digit());
    numbered.then_some(id)
}
