// DECISION: D037
use super::analyze;
use crate::structural_lint::rule_identities::{Kind, Reference, Signature, Structure};
use std::path::Path;

mod async_code;
mod configuration;
mod control_flow;
mod declarations;
mod errors;
mod markers;
mod ownership;
mod panic_surface;
mod reference;
mod shell_command;
mod signature;
mod std_forms;
mod test_code;
mod unsafe_site;

fn comment_lines(path: &str, source: &str) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::Comment))
        .map(|item| (item.line, item.symbol))
        .collect();
    lines.sort();
    lines
}

// DECISION: D219
#[test]
fn rust_accepts_only_exact_references_and_decision_exceptions() {
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
// DECISION: D001 panic-path
// INVARIANT: I002 panic-path
fn run() { let text = \"// not a comment\"; } // trailing
";
    let lines: Vec<_> = comment_lines("source.rs", source)
        .into_iter()
        .map(|(line, _)| line)
        .collect();
    assert_eq!(lines, [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 14]);
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

fn symbol_actuals(kind: Kind, path: &str, source: &str) -> Vec<(String, usize)> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut actuals: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == kind)
        .map(|item| (item.symbol, item.actual))
        .collect();
    actuals.sort();
    actuals
}

// DECISION: D122
fn line_actuals(kind: Kind, path: &str, source: &str) -> Vec<(usize, usize)> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut counts: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == kind)
        .map(|item| (item.line, item.actual))
        .collect();
    counts.sort();
    counts
}

// DECISION: D062
fn boolean_params(path: &str, source: &str) -> Vec<String> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut names: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Signature(Signature::BooleanParameter))
        .map(|item| item.symbol)
        .collect();
    names.sort();
    names
}

#[test]
fn rust_flags_bool_parameters_not_fields_or_other_types() {
    let source = "fn flat(count: u32, name: &str) {}
fn toggled(staged: bool, count: u32) {}
fn pair(first: bool, second: bool) {}
fn wrapped(maybe: Option<bool>, borrowed: &bool) {}
struct Config {
    enabled: bool,
}
impl Config {
    fn set(&mut self, ready: bool) {}
}
fn with_closure() {
    let choose = |active: bool| active;
}
";
    assert_eq!(
        boolean_params("source.rs", source),
        [
            "active: bool",
            "first: bool",
            "ready: bool",
            "second: bool",
            "staged: bool",
        ]
    );
}

#[test]
fn python_flags_bool_annotations_and_true_false_defaults() {
    let source = "def flat(count, name):
    pass
def toggled(active: bool, count):
    pass
def defaulted(ready=True, name=''):
    pass
def mixed(verbose: bool = False):
    pass
def wrapped(maybe: Optional[bool]):
    pass
class Config:
    def set(self, dry: bool):
        pass
";
    assert_eq!(
        boolean_params("source.py", source),
        [
            "active: bool",
            "dry: bool",
            "ready=True",
            "verbose: bool = False"
        ]
    );
}

// DECISION: D067
fn secret_findings(path: &str, source: &str) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::HardcodedSecret))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}

#[test]
fn rust_flags_literals_matching_known_credential_formats() {
    let aws = format!("AKIA{}", "0123456789ABCDEF");
    let github = format!("ghp_{}", "a".repeat(36));
    let pem = format!("{}BEGIN PRIVATE KEY{}", "-----", "-----");
    let source = format!(
        "fn settings() {{\n    let a = \"{aws}\";\n    let b = \"{github}\";\n    let c = \"{pem}\";\n    let d = \"ordinary value\";\n}}\n"
    );
    assert_eq!(
        secret_findings("source.rs", &source),
        [
            (2, "aws-access-key".into()),
            (3, "github-token".into()),
            (4, "private-key".into()),
        ]
    );
}

#[test]
fn rust_measures_lines_spanned_by_each_string_literal() {
    let source = r##"const QUERY: &str = r#"
SELECT id
FROM report
"#;
fn render(name: &str) -> String {
    let short = "one line";
    let bytes = b"first
second";
    format!("Hello {name},
welcome
back") + short
}
"##;
    assert_eq!(
        line_actuals(Kind::Structure(Structure::EmbeddedText), "text.rs", source),
        [(1, 4), (6, 1), (7, 2), (9, 3)]
    );
}

#[test]
fn rust_skips_string_literals_in_test_code() {
    let source = r##"fn shipped() -> &'static str {
    "kept
here"
}
#[cfg(test)]
mod tests {
    const EXPECTED: &str = "a
b
c";
    #[test]
    fn output() {
        assert_eq!(super::shipped(), "kept
here");
    }
}
#[test]
fn standalone() {
    let _ = "x
y";
}
"##;
    assert_eq!(
        line_actuals(Kind::Structure(Structure::EmbeddedText), "text.rs", source),
        [(2, 2)]
    );
}
