use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::{Kind, Ownership};
use std::path::Path;

// DECISION: D092
fn raii_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Ownership(Ownership::DisabledRaii))
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_disabled_raii_outside_tests() {
    let source = "fn run() {
    handle.keep();
    dir.into_path();
    mem::forget(guard);
    std::mem::forget(other);
    ManuallyDrop::new(value);
    Box::leak(boxed);
    config.leak();
    thing.into_inner();
    let s = \"mem::forget(x) in a string\";
}
#[cfg(test)]
mod tests {
    fn check() {
        handle.keep();
        mem::forget(guard);
    }
}
";
    assert_eq!(raii_findings("source.rs", source), [2, 3, 4, 5, 6, 7]);
}
