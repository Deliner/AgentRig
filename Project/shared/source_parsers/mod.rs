mod decision_markers;
mod secrets;

use anyhow::{Context, Result};
use std::path::Path;
use tree_sitter::{Language, Parser, Tree};

pub use decision_markers::reference;
pub use secrets::secret;

pub fn parse(path: &Path, source: &str) -> Result<Option<Tree>> {
    let language: Language = match path.extension().and_then(|extension| extension.to_str()) {
        // DECISION: D112
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
