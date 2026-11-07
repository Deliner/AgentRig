mod manifest;
mod measure;
mod python;
mod registry;
mod rust;
#[cfg(test)]
mod tests;
pub use measure::{Measurement, text};
pub use registry::{HANDLERS, handler};

use anyhow::{Context, Result};
use std::path::Path;

pub struct Analysis {
    pub measurements: Vec<Measurement>,
    pub parse_error: Option<usize>,
}
pub fn analyze(path: &Path, source: &str) -> Result<Analysis> {
    let handler = handler(path).context("unsupported source language")?;
    let inspect_node = match &handler.inspect {
        registry::Inspect::Document(analyze) => return Ok(analyze(source)),
        registry::Inspect::Tree(inspect) => *inspect,
    };
    let tree =
        crate::shared::syntax::parse(path, source)?.context("unsupported source language")?;
    let mut analysis = Analysis {
        measurements: Vec::new(),
        parse_error: None,
    };
    let valid = !tree.root_node().has_error();
    let mut pending = vec![tree.root_node()];
    while let Some(node) = pending.pop() {
        let invalid = node.is_error() || node.is_missing();
        if invalid {
            analysis
                .parse_error
                .get_or_insert(node.start_position().row + 1);
        }
        let inspect = valid && node.is_named();
        if inspect {
            inspect_node(node, source, &mut analysis.measurements);
        }
        let mut cursor = node.walk();
        pending.extend(
            node.children(&mut cursor)
                .collect::<Vec<_>>()
                .into_iter()
                .rev(),
        );
    }
    Ok(analysis)
}
