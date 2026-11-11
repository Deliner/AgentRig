use super::measured;
use crate::structural_lint::rules::reference::Reference;

// DECISION: D169
#[test]
fn rust_flags_should_panic_without_an_expected_message() {
    let source = "#[test]
#[should_panic]
fn rejects_empty_input() {
    parse(\"\").unwrap();
}
#[tokio::test]
#[should_panic()]
async fn rejects_async() {
    run().await;
}
";
    assert_eq!(
        measured(source, Reference::UntargetedPanic),
        [
            (2, "should_panic".to_string()),
            (7, "should_panic".to_string())
        ]
    );
}

#[test]
fn rust_skips_should_panic_with_an_expected_message_or_a_reference() {
    let source = "#[test]
#[should_panic(expected = \"Empty\")]
fn names_the_message() {
    parse(\"\").unwrap();
}
#[test]
#[should_panic = \"Empty\"]
fn names_it_by_value() {
    parse(\"\").unwrap();
}
#[test]
// DECISION: D001
#[should_panic]
fn borrows_twice() {
    cell.borrow_mut();
}
";
    assert!(measured(source, Reference::UntargetedPanic).is_empty());
}
