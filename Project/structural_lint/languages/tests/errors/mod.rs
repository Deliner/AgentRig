mod collapsed_chain;
mod discarded_result;
mod error_verdict;
mod except_pass;
mod process_success;

use crate::structural_lint::languages::{analyze, tests::DETACHED};
use crate::structural_lint::rule_identities::{Kind, Reference};
use std::path::Path;

// DECISION: D097
fn silent_findings(path: &str, source: &str) -> Vec<String> {
    let analysis = analyze(Path::new(path), source, DETACHED).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::SilentError))
        .collect();
    found.sort_by_key(|item| item.line);
    found
        .into_iter()
        .map(|item| format!("{}:{}", item.line, item.symbol))
        .collect()
}
