use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use std::path::Path;

// DECISION: D088
fn record_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Signature(Signature::TupleAsRecord))
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_tuple_parameters_and_returns_outside_tests() {
    let source = "fn packed(options: (bool, u32)) {}
fn returns_pair() -> (u32, u32) { (1, 2) }
// DECISION: D001
fn annotated(options: (bool, u32)) {}
fn plain(count: u32, name: &str) -> u32 { count }
fn unit() -> () {}
fn wrapped(maybe: Option<(u32, u32)>) {}
#[cfg(test)]
mod tests {
    fn helper(options: (bool, u32)) {}
}
";
    assert_eq!(record_findings("source.rs", source), [1, 2]);
}

// DECISION: D101
#[test]
fn rust_flags_unnamed_records_nested_in_generic_and_slice_types() {
    let source = "fn rows() -> Vec<(String, u64, bool)> { Vec::new() }
// DECISION: D001
fn annotated() -> Vec<(String, u64, bool)> { Vec::new() }
fn pairs() -> Vec<(String, u64)> { Vec::new() }
fn domain() -> Vec<(PathBuf, Config, Rules)> { Vec::new() }
fn slice(rows: &[(String, u64, bool)]) {}
fn deep(rows: Option<&mut Vec<(u8, u8, u8)>>) {}
fn referenced(row: &(String, u64, bool)) {}
type Rows = Vec<(String, u64, bool)>;
struct Table {
    rows: Vec<(String, u64, bool)>,
}
fn apply(rows: Vec<Row>) {
    let keep = |items: Vec<(&str, usize, char)>| items.len();
}
#[cfg(test)]
mod tests {
    fn rows() -> Vec<(String, u64, bool)> { Vec::new() }
}
";
    assert_eq!(record_findings("source.rs", source), [1, 6, 7, 14]);
}
