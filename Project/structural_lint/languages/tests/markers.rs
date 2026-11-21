use super::super::analyze;
use crate::structural_lint::rule_identities::{Kind, Reference};
use std::path::Path;

// DECISION: D074
fn disabled_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::DisabledTest))
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_ignore_attributes() {
    let source = "#[test]
#[ignore]
fn skipped() {}
#[test]
#[ignore = \"slow\"]
fn skipped_with_reason() {}
#[test]
fn active() {}
#[derive(Debug)]
struct Config;
";
    assert_eq!(disabled_findings("source.rs", source), [2, 5]);
}

#[test]
fn python_flags_skip_markers() {
    let source = "import pytest

@pytest.mark.skip(reason=\"flaky\")
def test_one():
    pass

@pytest.mark.skipif(True)
def test_three():
    pass

@pytest.mark.parametrize(\"value\", [1, 2])
def test_four(value):
    pass
";
    assert_eq!(disabled_findings("source.py", source), [3, 7]);
}

// DECISION: D080
fn allow_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::AllowJustification))
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_allow_attributes() {
    let source = "#[allow(dead_code)]
fn bare() {}
#[derive(Debug)]
struct Config;
#[test]
fn active() {}
mod inner {
    #![allow(unused)]
}
fn fixture() {
    let text = \"#[allow(dead_code)] still a string\";
}
";
    assert_eq!(allow_findings("source.rs", source), [1, 8]);
}

// DECISION: D098
#[test]
fn rust_flags_expect_and_conditional_suppressions_like_allow() {
    let source = "#[expect(dead_code)]
fn bare() {}
#[expect(unused, reason = \"a reason is not a reference\")]
fn reasoned() {}
#[cfg_attr(feature = \"strict\", allow(dead_code))]
fn conditional_allow() {}
#[cfg_attr(test, deny(warnings), expect(unused))]
fn later_argument() {}
#[cfg_attr(feature = \"serde\", derive(Serialize))]
fn unrelated() {}
mod inner {
    #![cfg_attr(not(test), allow(dead_code))]
}
#[cfg_attr(miri, ignore)]
fn skipped_conditionally() {}
#[test]
#[ignore]
fn skipped() {}
";
    assert_eq!(allow_findings("source.rs", source), [1, 3, 5, 7, 12]);
    assert_eq!(disabled_findings("source.rs", source), [14, 17]);
}

// DECISION: D081
fn debug_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::DebugArtifact))
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

// DECISION: D082
fn panic_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::PanicPath))
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
    panic!(\"boom\");
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
    assert_eq!(panic_findings("source.rs", source), [2, 3, 4, 5, 6, 7]);
}

#[test]
fn rust_test_context_exempts_only_real_test_code() {
    let source = "#[cfg(not(test))]
fn production() {
    value.unwrap();
}
#[cfg(feature = \"latest\")]
fn gated() {
    value.unwrap();
}
#[cfg(all(unix, test))]
fn unix_test_only() {
    value.unwrap();
}
#[cfg(test)]
mod tests {
    fn helper() {
        value.unwrap();
    }
}
#[tokio::test]
async fn async_check() {
    value.unwrap();
}
";
    assert_eq!(panic_findings("source.rs", source), [3, 7]);
}

#[test]
fn rust_flags_panic_paths_inside_macro_bodies() {
    let source = "fn run() {
    println!(\"{}\", value.unwrap());
    let markup = html! { <div>{ bad.unwrap() }</div> };
}
#[cfg(test)]
mod tests {
    fn check() {
        assert_eq!(value.unwrap(), 1);
    }
}
";
    assert_eq!(panic_findings("source.rs", source), [2]);
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
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::DebugAssertion))
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
    assert_eq!(assertion_findings("source.rs", source), [2, 3, 4]);
}

// DECISION: D085
fn wildcard_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::WildcardImport))
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
fn python_flags_wildcard_imports() {
    let source = "from os import path
from sys import *
from collections import abc
";
    assert_eq!(wildcard_findings("source.py", source), [2]);
}
