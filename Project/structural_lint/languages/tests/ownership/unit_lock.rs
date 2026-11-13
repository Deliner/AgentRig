use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::ownership::Ownership;
use std::path::Path;

// DECISION: D155
fn unit_lock_findings(source: &str) -> Vec<String> {
    let analysis = analyze(Path::new("source.rs"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Ownership(Ownership::UnitLock))
        .collect();
    found.sort_by_key(|item| item.line);
    found
        .into_iter()
        .map(|item| format!("{}:{}", item.line, item.symbol))
        .collect()
}

#[test]
fn rust_flags_unit_lock_types_and_constructors_whose_binding_infers_the_type() {
    let source = "use std::sync::{Arc, Mutex, RwLock};
struct Store {
    guard: Mutex<()>,
    handle: Arc<Mutex<()>>,
    data: Mutex<Vec<u8>>,
}
static WRITE: std::sync::RwLock<()> = RwLock::new(());
fn run() {
    let lock = Mutex::new(());
    let shared = Arc::new(RwLock::new(()));
    let typed: Mutex<()> = Mutex::new(());
    let value = Mutex::new(0);
    // DECISION: D001
    let justified = Mutex::new(());
    let text = \"Mutex<()>\";
    take(Mutex::new(()));
}
#[cfg(test)]
mod tests {
    fn check() {
        let lock = Mutex::new(());
    }
}
";
    assert_eq!(
        unit_lock_findings(source),
        [
            "3:Mutex",
            "4:Mutex",
            "7:RwLock",
            "9:Mutex",
            "10:RwLock",
            "11:Mutex"
        ]
    );
}
