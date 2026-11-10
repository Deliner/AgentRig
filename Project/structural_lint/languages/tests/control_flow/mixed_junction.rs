use crate::structural_lint::languages::analyze;
use std::path::Path;

// DECISION: D156
fn mixed_lines(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    analysis.mixed_junctions
}

#[test]
fn rust_reports_each_outermost_expression_joining_both_junctions_through_parentheses() {
    let source = "fn ready() -> bool {
    let mixed = active && !banned || admin;
    let deep = a || b && c || d;
    let grouped = (a && b) || (c && d);
    let negated = x && !(a || b);
    let single = a && b && c;
    if let Some(user) = session && user.active || admin {}
    if let Some(a) = left && (a.ready || forced) {}
    work(a || b, c && d);
    false
}
";
    assert_eq!(mixed_lines("source.rs", source), [2, 3, 4, 7, 8]);
}

#[test]
fn python_reports_each_outermost_expression_joining_and_with_or_through_parentheses() {
    let source = "def ready():
    mixed = active and not banned or admin
    grouped = (a and b) or (c and d)
    single = a or b or c
    negated = x and not (a or b)
    return x and (y or z)
";
    assert_eq!(mixed_lines("source.py", source), [2, 3, 6]);
}
