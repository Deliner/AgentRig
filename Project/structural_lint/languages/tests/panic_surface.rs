use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use std::path::Path;

const DEBUG_ASSERTION: Kind = Kind::Reference(Reference::DebugAssertion);
const PANIC_PATH: Kind = Kind::Reference(Reference::PanicPath);
const UNWIND: Kind = Kind::Reference(Reference::UnwindJustification);

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
    assert_eq!(lines(DEBUG_ASSERTION, source), [3, 5, 13, 24]);
    assert_eq!(lines(PANIC_PATH, source), Vec::<usize>::new());
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
    let found = findings(PANIC_PATH, source);
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
    let found = findings(UNWIND, source);
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
    let found = findings(PANIC_PATH, source);
    assert_eq!(found, [(4, "unwrap".to_string())]);
}

// DECISION: D147
#[test]
fn rust_referenced_clippy_allow_on_an_enclosing_item_covers_its_panics() {
    let source = "// DECISION: D001
#[allow(clippy::expect_used)]
fn covered() {
    value.expect(\"a\");
    other.expect_err(\"b\");
    println!(\"{}\", value.expect(\"c\"));
    value.unwrap();
}
// DECISION: D001
#[allow(clippy::unwrap_used)]
fn other_lint() {
    value.expect(\"d\");
}
#[allow(clippy::expect_used)]
fn unreferenced() {
    value.expect(\"e\");
}
// INVARIANT: I001
#[expect(clippy::panic, clippy::unreachable)]
mod checked {
    impl Thing {
        fn run(&self) {
            unreachable!();
            panic!(\"g\");
            todo!();
        }
    }
}
";
    assert_eq!(lines(PANIC_PATH, source), [7, 12, 16, 25]);
}

// DECISION: D148
#[test]
fn rust_flags_unjustified_panics_inside_macro_rules_bodies() {
    let source = "macro_rules! read {
    ($values:expr, $($field:ident),+) => {
        let first = $values.next().unwrap();
        // DECISION: D001
        let second = $values.next().expect(\"b\");
        $( let $field = load(stringify!($field)).unwrap_err(); )+
        writeln!(text, \"{}\", 1).unwrap();
        std::panic::resume_unwind(payload);
        panic!(\"c\");
        todo![];
        let kept = $values.unwrap_or(0);
        self.resume_unwind(payload);
        let field = value.unwrap;
        let name = unreachable;
    };
}
";
    let found = findings(PANIC_PATH, source);
    let expected = [
        (3, "unwrap"),
        (6, "unwrap_err"),
        (7, "unwrap"),
        (8, "resume_unwind"),
        (9, "panic"),
        (10, "todo"),
    ]
    .map(|(line, symbol)| (line, symbol.to_string()));
    assert_eq!(found, expected);
}

// DECISION: D148
#[test]
fn rust_skips_macro_rules_bodies_in_test_code_or_under_a_referenced_clippy_allow() {
    let source = "// DECISION: D001
#[allow(clippy::unwrap_used)]
macro_rules! allowed {
    () => { value.unwrap() };
}
#[cfg(test)]
mod tests {
    macro_rules! check {
        () => { value.unwrap() };
    }
}
#[cfg(test)]
macro_rules! helper {
    () => { value.unwrap() };
}
macro_rules! open {
    () => { value.unwrap() };
}
fn run() {
    call!(value.unwrap());
}
";
    assert_eq!(lines(PANIC_PATH, source), [17, 20]);
}
