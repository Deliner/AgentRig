use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use std::path::Path;

fn silenced(source: &str) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::SilencedBinding))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}

#[test]
fn rust_flags_a_wildcard_let_of_a_variable_that_silences_its_unused_warning() {
    let source = "fn handle(request: Request, unused: u32) {
    let _ = unused;
    let _ = request;
}
";
    assert_eq!(
        silenced(source),
        [(2, "unused".into()), (3, "request".into())]
    );
}

#[test]
fn rust_skips_constants_typed_computed_named_justified_and_test_bindings() {
    let source = "fn handle(request: Request) {
    let _ = MAX;
    let _: u32 = request;
    let _ = request.send();
    let _guard = request;
    // DECISION: D001
    let _ = request;
}
#[cfg(test)]
mod tests {
    fn check(value: u32) {
        let _ = value;
    }
}
";
    assert!(silenced(source).is_empty());
}
