mod dotted_extension;
mod eager_fallback;
mod epsilon_tolerance;
mod flag_filter;
mod indexed_array;
mod max_fallback;
mod superseded_form;
use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::{Kind, Reference};
use std::path::Path;

fn measured(source: &str, kind: Reference) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(kind))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}
