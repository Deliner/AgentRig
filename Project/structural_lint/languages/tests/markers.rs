use super::super::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::ownership::Ownership;
use crate::structural_lint::rules::reference::Reference;
use std::path::Path;

// DECISION: D074
fn disabled_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
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
        .filter(|item| item.kind == Kind::Reference(Reference::AllowJustification))
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

// DECISION: D098
#[test]
fn rust_flags_expect_and_conditional_suppressions_like_allow() {
    let source = "#[expect(dead_code)]
fn bare() {}
// DECISION: D001
#[expect(dead_code, reason = \"generated\")]
fn justified_expect() {}
#[expect(unused, reason = \"a reason is not a reference\")]
fn reasoned() {}
#[cfg_attr(feature = \"strict\", allow(dead_code))]
fn conditional_allow() {}
#[cfg_attr(test, deny(warnings), expect(unused))]
fn later_argument() {}
// INVARIANT: I002
#[cfg_attr(miri, allow(unused))]
fn justified_conditional() {}
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
    assert_eq!(allow_findings("source.rs", source), [1, 6, 8, 10, 18]);
    assert_eq!(disabled_findings("source.rs", source), [20, 23]);
}

// DECISION: D081
fn debug_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
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
    let analysis = analyze(Path::new(path), source).unwrap();
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
    panic_wrap!(
        // DECISION: D001
        guard.expect(\"ok\")
    );
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
    let analysis = analyze(Path::new(path), source).unwrap();
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

// DECISION: D089
fn eager_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
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
    // DECISION: D001
    let b = maybe().unwrap_or(fallback()?);
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

// DECISION: D094
#[test]
fn rust_flags_eager_fallback_inside_macro_bodies() {
    let source = "fn run() -> Result<u32, Error> {
    let a = json!(maybe().unwrap_or(fallback()?));
    let b = json!(
        // DECISION: D001
        maybe().unwrap_or(fallback()?)
    );
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

// DECISION: D090
fn discarded_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::DiscardedResult))
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_discarded_result_statements_outside_tests() {
    let source = "fn run() {
    thing.ok();
    // DECISION: D001
    other.ok();
    let kept = thing.ok();
    saved = thing.ok();
    thing.ok()?;
    thing.run();
}
#[cfg(test)]
mod tests {
    fn check() {
        value.ok();
    }
}
";
    assert_eq!(discarded_findings("source.rs", source), [2]);
}

// DECISION: D099
#[test]
fn rust_flags_propagated_status_and_output_statements_that_drop_the_outcome() {
    let source = "fn run() -> io::Result<()> {
    Command::new(\"git\").status()?;
    // DECISION: D001
    Command::new(\"git\").output()?;
    command(root, args)
        .args([\"-c\"])
        .output()?;
    let status = Command::new(\"git\").status()?;
    Command::new(\"git\").status()?.success();
    check(Command::new(\"git\").output()?);
    writer.output(data)?;
    Command::new(\"git\").status();
    thing.ok()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    fn check() {
        Command::new(\"git\").status()?;
    }
}
";
    assert_eq!(discarded_findings("source.rs", source), [2, 5]);
}

// DECISION: D114
#[test]
fn rust_flags_discarded_option_combinator_and_err_statements() {
    let source = "fn run(value: Option<u8>) -> Option<u8> {
    value.map(|item| store(item));
    value.and_then(|item| load(item));
    value.map_or(0, |item| send(item));
    value.map_or_else(|| 0, |item| send(item));
    outcome.err();
    // DECISION: D001
    value.map(|item| store(item));
    let kept = value.map(|item| store(item));
    saved = value.and_then(|item| load(item));
    check(value.map(|item| store(item)));
    value.map(|item| store(item))?;
    value.filter(|item| *item > 0);
    value.map(|item| item + 1)
}
#[cfg(test)]
mod tests {
    fn check() {
        value.map(|item| store(item));
    }
}
";
    assert_eq!(discarded_findings("source.rs", source), [2, 3, 4, 5, 6]);
}

// DECISION: D092
fn raii_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
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
    // DECISION: D001
    other.keep();
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
