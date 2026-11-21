mod disabled_raii;
mod exposed_guard;
mod global_state;
mod paired_calls;
mod recursive_deletion;
mod unit_lock;
mod wrapper;
use crate::structural_lint::languages::{Context, analyze, tests::DETACHED};
use crate::structural_lint::rule_identities::{Kind, Ownership, Pair};
use std::path::Path;

fn measured(source: &str, pairs: &[Pair], kind: Ownership) -> Vec<(usize, String)> {
    let context = Context { pairs, ..DETACHED };
    let analysis = analyze(Path::new("source.rs"), source, context).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Ownership(kind))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}
