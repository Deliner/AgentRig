// DECISION: D037
use super::analyze;
use crate::structural_lint::rules::kind::Kind;
use std::path::Path;

fn comment_lines(path: &str, source: &str) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::CommentReference)
        .map(|item| (item.line, item.symbol))
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_accepts_only_exact_decision_and_invariant_references() {
    let source = "// DECISION: D001
// INVARIANT: I002
// plain
/// documentation
//! module documentation
/* DECISION: D001 */
// DECISION: D001 -- reason
//DECISION: D001
// DECISION: D01
// NOTE: D001
// SAFETY: the caller checks bounds
fn run() { let text = \"// not a comment\"; } // trailing
";
    let lines: Vec<_> = comment_lines("source.rs", source)
        .into_iter()
        .map(|(line, _)| line)
        .collect();
    assert_eq!(lines, [3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
}

#[test]
fn python_rejects_other_comments_and_lone_strings() {
    let source = "\"\"\"Module documentation.\"\"\"
# DECISION: D001
# INVARIANT: I002
# plain
# type: ignore
value = 1  # noqa: E501
text = \"# not a comment\"  # trailing
def run():
    \"\"\"Documentation.\"\"\"
    return '''value'''
";
    assert_eq!(
        comment_lines("source.py", source),
        [
            (1, "docstring".into()),
            (4, "comment".into()),
            (5, "comment".into()),
            (6, "comment".into()),
            (7, "comment".into()),
            (9, "docstring".into()),
        ]
    );
}

// DECISION: D061
fn nesting_depths(path: &str, source: &str) -> Vec<(String, u64)> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut depths: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::NestingDepth)
        .map(|item| (item.symbol, item.actual))
        .collect();
    depths.sort();
    depths
}

#[test]
fn rust_nesting_counts_blocks_without_else_if_or_nested_items() {
    let source = "fn flat() {
    let value = 1;
}
fn nested() {
    for value in items {
        if value > 0 {
            while value > 0 {
                work();
            }
        }
    }
}
fn ladder(choice: i32) {
    if choice == 0 {
    } else if choice == 1 {
    } else {
    }
}
fn outer() {
    let closure = |value: i32| {
        if value > 0 {
            work();
        }
    };
}
";
    assert_eq!(
        nesting_depths("source.rs", source),
        [
            ("<anonymous>".into(), 1),
            ("flat".into(), 0),
            ("ladder".into(), 1),
            ("nested".into(), 3),
            ("outer".into(), 0),
        ]
    );
}

#[test]
fn python_nesting_counts_blocks_without_elif_or_nested_functions() {
    let source = "def flat():
    value = 1
def nested():
    for value in items:
        if value > 0:
            while value > 0:
                work()
def ladder(choice):
    if choice == 0:
        pass
    elif choice == 1:
        pass
    else:
        pass
def outer():
    def inner():
        if ready:
            work()
";
    assert_eq!(
        nesting_depths("source.py", source),
        [
            ("flat".into(), 0),
            ("inner".into(), 1),
            ("ladder".into(), 1),
            ("nested".into(), 3),
            ("outer".into(), 0),
        ]
    );
}
