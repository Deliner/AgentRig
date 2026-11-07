// DECISION: D037
use super::analyze;
use crate::structural_lint::rules::kind::Kind;
use std::path::Path;

mod global_state;
mod manifest;
mod markers;
mod panic_surface;
mod recursive_deletion;
mod silent_error;
mod super_reach;
mod tuple_as_record;
mod unsafe_site;

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

// DECISION: D062
fn boolean_params(path: &str, source: &str) -> Vec<String> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut names: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::BooleanParameter)
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

// DECISION: D063
fn branch_budgets(path: &str, source: &str) -> Vec<(String, u64)> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut budgets: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::BranchBudget)
        .map(|item| (item.symbol, item.actual))
        .collect();
    budgets.sort();
    budgets
}

#[test]
fn rust_counts_ifs_and_matches_within_each_function() {
    let source = "fn flat() {
    let value = 1;
}
fn ladder(choice: i32) {
    if choice == 0 {
    } else if choice == 1 {
    } else {
    }
}
fn dispatch(choice: i32) -> i32 {
    match choice {
        0 => 10,
        1 => 20,
        _ => 30,
    }
}
fn outer(choice: i32) {
    let closure = |value: i32| {
        if value > 0 {
            work();
        }
    };
    if choice > 0 {
        work();
    }
}
";
    assert_eq!(
        branch_budgets("source.rs", source),
        [
            ("<anonymous>".into(), 1),
            ("dispatch".into(), 1),
            ("flat".into(), 0),
            ("ladder".into(), 2),
            ("outer".into(), 1),
        ]
    );
}

#[test]
fn python_counts_ifs_elifs_ternaries_and_matches_within_each_function() {
    let source = "def flat():
    value = 1
def ladder(choice):
    if choice == 0:
        pass
    elif choice == 1:
        pass
    else:
        pass
def dispatch(choice):
    match choice:
        case 0:
            return 10
        case _:
            return 20
def ternary(choice):
    return 1 if choice else 2
def outer(choice):
    def inner(value):
        if value:
            pass
    if choice:
        pass
";
    assert_eq!(
        branch_budgets("source.py", source),
        [
            ("dispatch".into(), 1),
            ("flat".into(), 0),
            ("inner".into(), 1),
            ("ladder".into(), 2),
            ("outer".into(), 1),
            ("ternary".into(), 1),
        ]
    );
}

// DECISION: D067
fn secret_findings(path: &str, source: &str) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::HardcodedSecret)
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}

// DECISION: D069
fn silent_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::SilentError)
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn python_flags_except_pass_without_a_justifying_reference() {
    let source = "def guarded():
    try:
        work()
    except ValueError:
        pass
    try:
        work()
    except KeyError:
        # DECISION: D001
        pass
    try:
        work()
    except OSError as error:
        log(error)
    try:
        work()
    except TypeError:
        cleanup()
        pass
    try:
        work()
    except IndexError: pass
";
    assert_eq!(silent_findings("source.py", source), [4, 22]);
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
