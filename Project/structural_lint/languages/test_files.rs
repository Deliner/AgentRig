use super::neutral_metrics::Measurement;
use crate::structural_lint::rule_identities::{Kind, Structure};
use std::path::Path;

// DECISION: D223
pub(super) fn rust(name: &str) -> bool {
    name == "tests.rs"
}
// DECISION: D223
pub(super) fn python(name: &str) -> bool {
    name.strip_suffix(".py")
        .is_some_and(|stem| stem.starts_with("test_") || stem.ends_with("_test"))
}
// DECISION: D223
pub(super) fn misplaced(path: &Path, test_file: Option<fn(&str) -> bool>) -> Vec<Measurement> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let placed = path.components().any(|part| part.as_os_str() == "tests");
    let outside = test_file.is_some_and(|test_file| test_file(name)) && !placed;
    let measurement = outside.then(|| Measurement {
        kind: Kind::Structure(Structure::TestPlacement),
        line: 1,
        symbol: name.into(),
        actual: 1,
    });
    measurement.into_iter().collect()
}
