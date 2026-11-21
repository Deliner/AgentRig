mod constant_default;
mod derived_trait;
mod foreign_signature;
mod implicit_order;
mod ord_equality;
mod partial_impl;
mod python_override;
mod same_type;
mod tuple_as_record;
mod type_shape;
mod untyped_conversion;
use crate::structural_lint::languages::{Context, Project, analyze, tests::DETACHED};
use crate::structural_lint::rule_identities::Kind;
use std::path::Path;

fn measured_within(
    path: &Path,
    source: &str,
    project: &dyn Project,
    kinds: &[Kind],
) -> Vec<(usize, String)> {
    let context = Context {
        pairs: &[],
        project,
    };
    let analysis = analyze(path, source, context).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| kinds.contains(&item.kind))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}

fn measured(source: &str, kind: Kind) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source, DETACHED).unwrap();
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
