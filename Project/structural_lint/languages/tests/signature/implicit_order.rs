use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use std::path::Path;

// DECISION: D155
fn implicit_order_findings(source: &str) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Signature(Signature::ImplicitOrder))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}

#[test]
fn rust_flags_ordered_enums_whose_variants_order_by_declaration() {
    let source = "#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    Low,
    High,
}
#[derive(PartialEq, PartialOrd)]
pub enum Mixed {
    First = 1,
    Second,
}
#[cfg_attr(feature = \"order\", derive(PartialEq, Eq, PartialOrd, Ord))]
enum Conditional {
    Before,
    After,
}
";
    assert_eq!(
        implicit_order_findings(source),
        [
            (2, "Level".to_string()),
            (7, "Mixed".to_string()),
            (12, "Conditional".to_string()),
        ]
    );
}

#[test]
fn rust_skips_pinned_single_unordered_justified_and_test_enums() {
    let source = "#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Pinned {
    Low = 0,
    High = 1,
}
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Single {
    Only,
}
#[derive(Clone, PartialEq)]
enum Unordered {
    First,
    Second,
}
// DECISION: D001
#[derive(PartialEq, PartialOrd)]
enum Justified {
    First,
    Second,
}
#[cfg(test)]
mod tests {
    #[derive(PartialEq, PartialOrd)]
    enum Probe {
        First,
        Second,
    }
}
";
    assert!(implicit_order_findings(source).is_empty());
}
