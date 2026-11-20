use super::super::analyze;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::ownership::Ownership;
use crate::structural_lint::rule_identities::reference::Reference;
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

// DECISION: D089
fn eager_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::EagerFallback))
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_eager_fallback_arguments_that_propagate() {
    let source = "fn run() -> Result<u32, Error> {
    let a = maybe().unwrap_or(fallback()?);
    let c = maybe().ok_or(Error)?;
    let d = option.map_or(0, |value| compute(value)?);
    let e = option.unwrap_or(0);
    Ok(a)
}
#[cfg(test)]
mod tests {
    fn check() {
        let f = maybe().unwrap_or(fallback()?);
    }
}
";
    assert_eq!(eager_findings("source.rs", source), [2]);
}

// DECISION: D168
#[test]
fn rust_flags_eager_divisions_and_propagation_in_or_but_not_then_some() {
    let source = "fn run(total: u32, count: u32, ready: bool) -> Option<u32> {
    let a = maybe().unwrap_or(total / count);
    let b = maybe().unwrap_or(total / 2);
    let c = maybe().unwrap_or(pick(|| total / count));
    let d = ready.then_some(compute());
    let e = ready.then_some(format!(\"{total}\"));
    let f = ready.then_some(Wrapper(total));
    let g = ready.then_some(Kind::Ready(total));
    let h = ready.then_some(Wrapper(total.to_string()));
    let i = maybe().or(fallback()?);
    let j = maybe().or(cached());
    let k = maybe().unwrap_or(compute());
    a
}
";
    assert_eq!(eager_findings("source.rs", source), [2, 10]);
}

// DECISION: D094
#[test]
fn rust_flags_eager_fallback_inside_macro_bodies() {
    let source = "fn run() -> Result<u32, Error> {
    let a = json!(maybe().unwrap_or(fallback()?));
    let c = vec![compute()?, maybe().unwrap_or(0)];
    Ok(a)
}
#[cfg(test)]
mod tests {
    fn check() {
        let d = json!(maybe().unwrap_or(fallback()?));
    }
}
";
    assert_eq!(eager_findings("source.rs", source), [2]);
}

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
