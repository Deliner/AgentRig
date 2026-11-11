mod error_text;
mod silent_error;
mod unhandled_error;
use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use std::path::Path;

fn findings(reference: Reference, source: &str) -> Vec<String> {
    let analysis = analyze(Path::new("source.rs"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(reference))
        .collect();
    found.sort_by_key(|item| item.line);
    found
        .into_iter()
        .map(|item| format!("{}:{}", item.line, item.symbol))
        .collect()
}
