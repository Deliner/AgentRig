use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use std::path::Path;

// DECISION: D115
fn findings(kind: Kind, source: &str) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == kind)
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}
fn lines(kind: Kind, source: &str) -> Vec<usize> {
    findings(kind, source)
        .into_iter()
        .map(|(line, _)| line)
        .collect()
}

#[test]
fn rust_treats_asserts_compiled_only_with_debug_assertions_as_debug_assertions() {
    let source = "fn run(ready: bool) {
    #[cfg(debug_assertions)]
    assert!(ready);
    if cfg!(debug_assertions) {
        assert_eq!(a, b);
    } else {
        assert_ne!(a, b);
    }
    #[cfg(all(debug_assertions, unix))]
    {
        // DECISION: D001
        assert!(ready);
        assert!(ready);
    }
    #[cfg(not(debug_assertions))]
    assert!(ready);
    if cfg!(debug_assertions) && ready {
        assert!(ready);
    }
    assert!(ready);
}
#[cfg(debug_assertions)]
fn checked() {
    assert!(ready);
}
#[cfg(test)]
mod tests {
    #[cfg(debug_assertions)]
    fn check() {
        assert!(ready);
    }
}
";
    assert_eq!(lines(Kind::DebugAssertion, source), [3, 5, 13, 24]);
    assert_eq!(lines(Kind::PanicPath, source), [] as [usize; 0]);
}

#[test]
fn rust_flags_unwrap_err_expect_err_and_resume_unwind_as_panic_paths() {
    let source = "fn run() {
    let error = outcome.unwrap_err();
    outcome.expect_err(\"must fail\");
    std::panic::resume_unwind(payload);
    resume_unwind(payload);
    // DECISION: D001
    panic::resume_unwind(payload);
    log(format!(\"{}\", outcome.unwrap_err()));
    let kept = outcome.unwrap_or_else(|error| error);
    self.resume_unwind(payload);
}
#[cfg(test)]
mod tests {
    fn check() {
        outcome.unwrap_err();
    }
}
";
    let found = findings(Kind::PanicPath, source);
    let expected = [
        (2, "unwrap_err"),
        (3, "expect_err"),
        (4, "resume_unwind"),
        (5, "resume_unwind"),
        (8, "unwrap_err"),
    ]
    .map(|(line, symbol)| (line, symbol.to_string()));
    assert_eq!(found, expected);
}

#[test]
fn rust_flags_unjustified_panic_catches_outside_tests() {
    let source = "fn run() {
    let caught = std::panic::catch_unwind(|| work());
    let wrapped = panic::catch_unwind(AssertUnwindSafe(|| work()));
    // DECISION: D001
    let justified = catch_unwind(std::panic::AssertUnwindSafe(|| work()));
    let future = task.catch_unwind();
    let plain = std::panic::set_hook(hook);
    let named = UnwindSafe::check(value);
}
#[cfg(test)]
mod tests {
    fn check() {
        let caught = std::panic::catch_unwind(|| work());
    }
}
";
    let found = findings(Kind::UnwindJustification, source);
    let expected = [
        (2, "catch_unwind"),
        (3, "AssertUnwindSafe"),
        (3, "catch_unwind"),
        (6, "catch_unwind"),
    ]
    .map(|(line, symbol)| (line, symbol.to_string()));
    assert_eq!(found, expected);
}

// DECISION: D112
#[test]
fn rust_flags_panic_paths_beside_primitive_named_macros_and_lifetimes_first_after_dyn() {
    let source =
        "fn check(callback: Box<dyn 'static + Fn() + Send>) -> &'static (dyn 'static + Sync) {
    let expected = str![[r#\"done\"#]];
    let bytes = u8![1, 2];
    load(expected, bytes).unwrap()
}
";
    let found = findings(Kind::PanicPath, source);
    assert_eq!(found, [(4, "unwrap".to_string())]);
}
