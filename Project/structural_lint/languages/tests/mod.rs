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
fn run() { let text = \"// not a comment\"; } // trailing
";
    let lines: Vec<_> = comment_lines("source.rs", source)
        .into_iter()
        .map(|(line, _)| line)
        .collect();
    assert_eq!(lines, [3, 4, 5, 6, 7, 8, 9, 10, 11]);
}

#[test]
fn python_rejects_other_comments_and_lone_strings() {
    let source = "\"\"\"Module documentation.\"\"\"
# DECISION: D001
# INVARIANT: I002
# plain
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
            (7, "docstring".into()),
        ]
    );
}
