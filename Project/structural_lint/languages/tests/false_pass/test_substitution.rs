use super::measured;
use crate::structural_lint::rules::reference::Reference;

// DECISION: D169
#[test]
fn rust_flags_items_that_every_test_build_excludes() {
    let source = "#[cfg(not(test))]
fn now() -> SystemTime {
    SystemTime::now()
}
#[cfg(test)]
fn now() -> SystemTime {
    UNIX_EPOCH
}
#[cfg(all(not(test), unix))]
mod clock;
#[cfg(not(any(test, feature = \"mock\")))]
const TIMEOUT: u64 = 30;
impl Service {
    #[cfg(not(test))]
    fn connect(&self) {}
}
mod real {
    #![cfg(not(test))]
    fn run() {}
}
";
    assert_eq!(
        measured(source, Reference::TestSubstitution),
        [
            (1, "cfg".to_string()),
            (9, "cfg".to_string()),
            (11, "cfg".to_string()),
            (14, "cfg".to_string()),
            (18, "cfg".to_string()),
        ]
    );
}

#[test]
fn rust_skips_cfg_attr_partial_other_flag_statement_field_justified_and_test_items() {
    let source = "#![cfg_attr(not(test), no_std)]
#[cfg(any(not(test), feature = \"real\"))]
fn either() {}
#[cfg(not(debug_assertions))]
fn release() {}
#[cfg(any())]
fn never() {}
fn body() {
    #[cfg(not(test))]
    let value = 1;
}
struct Config {
    #[cfg(not(test))]
    path: String,
}
// DECISION: D001
#[cfg(not(test))]
fn justified() {}
#[cfg(test)]
mod tests {
    #[cfg(not(test))]
    fn unreachable() {}
}
";
    assert!(measured(source, Reference::TestSubstitution).is_empty());
}
