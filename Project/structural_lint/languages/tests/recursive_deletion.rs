use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::ownership::Ownership;
use std::path::Path;

// DECISION: D102
fn deletion_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Ownership(Ownership::RecursiveDeletion))
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_remove_dir_all_calls_from_any_source_outside_tests() {
    let source = "fn clean(root: &Path, name: &str) -> std::io::Result<()> {
    std::fs::remove_dir_all(root.join(name))?;
    fs::remove_dir_all(root)?;
    remove_dir_all(root)?;
    // DECISION: D001
    fs::remove_dir_all(root.join(name))?;
    self.store.remove_dir_all(root)?;
    fs::remove_file(root.join(name))?;
    fs::remove_dir(root.join(name))?;
    let pending = tokio::fs::remove_dir_all(root).await;
    assert!(fs::remove_dir_all(root).is_ok());
    Ok(())
}
#[cfg(test)]
mod tests {
    fn helper(root: &Path) { std::fs::remove_dir_all(root).unwrap(); }
}
";
    assert_eq!(deletion_findings("source.rs", source), [2, 3, 4, 7, 10, 11]);
}
