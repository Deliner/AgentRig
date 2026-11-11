use super::measured;
use crate::structural_lint::rules::reference::Reference;

#[test]
fn rust_flags_assert_of_a_bare_is_err_in_test_code() {
    let source = "#[cfg(test)]
mod tests {
    #[test]
    fn rejects_empty() {
        assert!(parse(\"\").is_err());
        assert!(parse(\"-\").is_err(), \"minus is not a number\");
    }
    fn helper() {
        assert!(load().is_err());
    }
}
#[test]
fn stands_alone() {
    assert!(std::fs::read(\"missing\").is_err());
}
";
    assert_eq!(
        measured(source, Reference::UntargetedError),
        [
            (5, "is_err".to_string()),
            (6, "is_err".to_string()),
            (9, "is_err".to_string()),
            (14, "is_err".to_string()),
        ]
    );
}

#[test]
fn rust_skips_production_negated_combined_other_and_justified_assertions() {
    let source = "fn check(input: &str) {
    assert!(parse(input).is_err());
}
#[test]
fn rejects_empty() {
    assert!(!parse(\"1\").is_err());
    assert!(parse(\"\").is_err() && parse(\"-\").is_err());
    assert!(parse(\"\").is_ok());
    assert!(parse(\"\").is_err_and(|error| error.empty()));
    debug_assert!(parse(\"\").is_err());
    // DECISION: D001
    assert!(parse(\"\").is_err());
}
";
    assert!(measured(source, Reference::UntargetedError).is_empty());
}
