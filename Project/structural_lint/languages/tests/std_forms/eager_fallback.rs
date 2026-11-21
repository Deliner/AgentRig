use crate::structural_lint::languages::{analyze, tests::DETACHED};
use crate::structural_lint::rule_identities::{Kind, Reference};
use std::path::Path;

// DECISION: D089
fn eager_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source, DETACHED).unwrap();
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
