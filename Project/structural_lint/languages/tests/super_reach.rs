use super::super::analyze;
use crate::structural_lint::rules::kind::Kind;
use std::path::Path;

// DECISION: D100
fn reach_findings(path: &str, source: &str) -> Vec<String> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut sites: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::SuperSuperReach)
        .map(|item| (item.line, item.symbol))
        .collect();
    sites.sort();
    sites
        .into_iter()
        .map(|(line, symbol)| format!("{line}:{symbol}"))
        .collect()
}

#[test]
fn rust_flags_paths_reaching_two_module_levels_up_outside_tests() {
    let source = "use super::super::analyze;
// DECISION: D001
use super::super::justified;
use super::item;
use crate::root::item;
use super::super::{
    first,
    second as alias,
};
fn run() -> super::super::Kind {
    let value = super::super::make(1);
    let one = super::make(1);
    let text = \"super::super::not_a_path\";
    super::super::Vec::<u8>::new()
}
#[cfg(test)]
mod tests {
    use super::super::analyze;
    fn check() {
        super::super::make(1);
    }
}
";
    assert_eq!(
        reach_findings("source.rs", source),
        [
            "1:analyze",
            "6:super::super",
            "10:Kind",
            "11:make",
            "14:Vec"
        ]
    );
}
