use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

#[test]
fn rust_flags_tests_whose_every_check_sits_in_an_if_without_else() {
    let source = "#[test]
fn parses() {
    if let Ok(value) = parse(\"1\") {
        assert_eq!(value, 1);
    }
}
#[tokio::test]
async fn fetches() {
    let result = fetch().await;
    if result.is_ok() {
        assert!(result.ready());
    } else if result.is_err() {
        assert_ne!(code, 0);
    }
}
#[test]
fn closures() {
    items.iter().for_each(|item| {
        if let Some(value) = item {
            assert!(value.ready());
        }
    });
}
";
    assert_eq!(
        measured(source, Reference::ConditionalCheck),
        [
            (2, "parses".into()),
            (8, "fetches".into()),
            (17, "closures".into()),
        ]
    );
}

#[test]
fn rust_skips_tests_with_a_check_in_an_else_branch_or_outside_any_if() {
    let source = "#[test]
fn checks_both() {
    if ready() {
        assert!(value());
    } else {
        assert!(other());
    }
}
#[test]
fn checks_once() {
    if let Some(value) = parse() {
        assert_eq!(value, 1);
    }
    assert!(done());
}
#[test]
fn bound() {
    let Some(value) = parse() else {
        panic!(\"no value\");
    };
    assert_eq!(value, 1);
}
";
    assert!(measured(source, Reference::ConditionalCheck).is_empty());
}

#[test]
fn rust_skips_tests_without_checks_helpers_and_justified_tests() {
    let source = "#[test]
fn no_checks() {
    if ready() {
        work();
    }
}
fn helper() {
    if ready() {
        assert!(value());
    }
}
// DECISION: D001
#[test]
fn justified() {
    if ready() {
        assert!(value());
    }
}
";
    assert!(measured(source, Reference::ConditionalCheck).is_empty());
}
