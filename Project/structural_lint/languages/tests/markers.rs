use super::super::analyze;
use crate::structural_lint::rules::kind::Kind;
use std::path::Path;

// DECISION: D074
fn disabled_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::DisabledTest)
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_ignore_without_a_reference_on_the_line_above() {
    let source = "#[test]
#[ignore]
fn skipped() {}
#[test]
// DECISION: D001
#[ignore]
fn justified_skip() {}
#[test]
#[ignore = \"slow\"]
fn skipped_with_reason() {}
#[test]
fn active() {}
#[derive(Debug)]
struct Config;
";
    assert_eq!(disabled_findings("source.rs", source), [2, 9]);
}

#[test]
fn python_flags_skip_markers_without_a_justifying_reference() {
    let source = "import pytest

@pytest.mark.skip(reason=\"flaky\")
def test_one():
    pass

# DECISION: D001
@pytest.mark.skip
def test_two():
    pass

@pytest.mark.skipif(True)
def test_three():
    pass

@pytest.mark.parametrize(\"value\", [1, 2])
def test_four(value):
    pass
";
    assert_eq!(disabled_findings("source.py", source), [3, 12]);
}

// DECISION: D080
fn allow_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::AllowJustification)
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_allow_without_a_reference_on_the_line_above() {
    let source = "#[allow(dead_code)]
fn bare() {}
// DECISION: D001
#[allow(dead_code)]
fn justified_outer() {}
#[derive(Debug)]
struct Config;
#[test]
fn active() {}
mod inner {
    #![allow(unused)]
    // INVARIANT: I002
    #![allow(clippy::all)]
}
fn fixture() {
    let text = \"#[allow(dead_code)] still a string\";
}
";
    assert_eq!(allow_findings("source.rs", source), [1, 11]);
}

// DECISION: D081
fn debug_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::DebugArtifact)
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

// DECISION: D082
fn panic_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::PanicPath)
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_panic_paths_outside_test_code() {
    let source = "fn run() {
    let a = thing.unwrap();
    thing.expect(\"m\");
    // DECISION: D001
    let b = other.unwrap();
    panic!(\"boom\");
    // INVARIANT: I001
    todo!();
    unimplemented!();
    unreachable!();
    let s = \"x.unwrap() in a string\";
}
#[cfg(test)]
mod tests {
    #[test]
    fn checks() {
        value.unwrap();
        panic!(\"ok\");
    }
}
";
    assert_eq!(panic_findings("source.rs", source), [2, 3, 6, 9, 10]);
}

#[test]
fn rust_flags_dbg_macro_invocations() {
    let source = "fn probe() {
    dbg!(compute());
    println!(\"{}\", 1);
    let _ = vec![1, 2];
    let text = \"dbg!(x) in a string\";
    log(dbg!(inner));
}
";
    assert_eq!(debug_findings("source.rs", source), [2, 6]);
}

// DECISION: D084
fn assertion_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::DebugAssertion)
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_debug_assertions_outside_test_code() {
    let source = "fn run() {
    debug_assert!(ready);
    debug_assert_eq!(a, b);
    // DECISION: D001
    debug_assert_ne!(a, b);
    assert!(always);
    assert_eq!(a, b);
    let s = \"debug_assert!(x) in a string\";
}
#[cfg(test)]
mod tests {
    #[test]
    fn checks() {
        debug_assert!(value);
    }
}
";
    assert_eq!(assertion_findings("source.rs", source), [2, 3]);
}

// DECISION: D085
fn wildcard_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::WildcardImport)
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_glob_imports_outside_test_code() {
    let source = "use crate::foo::*;
use crate::bar::{Baz, Qux};
use crate::grouped::{First, nested::*};
// DECISION: D001
use crate::justified::*;
fn run() {
    let text = \"use x::* in a string\";
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks() {}
}
";
    assert_eq!(wildcard_findings("source.rs", source), [1, 3]);
}

#[test]
fn python_flags_wildcard_imports_without_a_reference() {
    let source = "from os import path
from sys import *
# DECISION: D001
from json import *
from collections import abc
";
    assert_eq!(wildcard_findings("source.py", source), [2]);
}
