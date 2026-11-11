mod constant_default;
mod derived_trait;
mod implicit_order;
mod ord_equality;
mod partial_impl;
mod same_type;
mod tuple_as_record;
mod type_shape;
use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use std::path::Path;

fn measured(source: &str, kind: Kind) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == kind)
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}
