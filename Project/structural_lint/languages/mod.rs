mod flag_list;
mod manifest;
mod neutral_metrics;
mod program;
mod python;
// DECISION: D229
mod python_override;
mod registry;
mod rust;
// DECISION: D223
mod test_files;
#[cfg(test)]
mod tests;
pub use neutral_metrics::{
    Analysis, Code, Context, Imported, Measurement, Project, PythonImport, Source, text,
};
pub use program::{program_language, program_languages};
pub use registry::{HANDLERS, handler};
pub(crate) use rust::{ENTRY_FILES, in_test_context};

use crate::shared::source_parsers;
use anyhow::{Context as _, Result};
use std::path::Path;

// DECISION: D156
// DECISION: D229
pub fn analyze(path: &Path, source: &str, context: Context<'_>) -> Result<Analysis> {
    let handler = handler(path).context("unsupported source language")?;
    let inspect_node = match &handler.inspect {
        registry::Inspect::Document(analyze) => {
            let mut analysis = analyze(source);
            analysis.exceptions = source_parsers::line_exceptions(source);
            return Ok(analysis);
        }
        registry::Inspect::Tree(inspect) => *inspect,
    };
    let tree = source_parsers::parse(path, source)?.context("unsupported source language")?;
    let mut analysis = Analysis {
        exceptions: source_parsers::tree_exceptions(tree.root_node(), source),
        measurements: test_files::misplaced(path, handler.test_file),
        ..Analysis::default()
    };
    let valid = !tree.root_node().has_error();
    let code = Code::new(path, source);
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
            inspect_node(node, code, context, &mut analysis);
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
