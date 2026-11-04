use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use std::path::Path;

// DECISION: D097
fn silent_findings(path: &str, source: &str) -> Vec<String> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::SilentError)
        .collect();
    found.sort_by_key(|item| item.line);
    found
        .into_iter()
        .map(|item| format!("{}:{}", item.line, item.symbol))
        .collect()
}

#[test]
fn rust_flags_read_dir_chains_that_drop_failed_entries() {
    let source = "use std::fs;
fn list(root: &Path) -> Result<Vec<PathBuf>> {
    let direct: Vec<_> = fs::read_dir(root)?.flatten().map(|entry| entry.path()).collect();
    let owned: Vec<_> = std::fs::read_dir(root)?.into_iter().flatten().collect();
    let method: Vec<_> = root.read_dir()?.filter_map(Result::ok).collect();
    let closure: Vec<_> = read_dir(root)?.filter_map(|entry| entry.ok()).collect();
    let bounded: Vec<_> = fs::read_dir(root)?
        .map_while(Result::ok)
        .collect();
    // DECISION: D001
    let justified: Vec<_> = fs::read_dir(root)?.flatten().collect();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
    }
    let kept = fs::read_dir(root)?.map(|entry| entry.map(|found| found.path()));
    let unrelated: Vec<_> = lines.iter().flatten().collect();
    Ok(direct)
}
#[test]
fn probe() -> Result<()> {
    let count = fs::read_dir(\".\")?.flatten().count();
    Ok(())
}
";
    assert_eq!(
        silent_findings("source.rs", source),
        [
            "3:flatten",
            "4:flatten",
            "5:filter_map",
            "6:filter_map",
            "7:map_while"
        ]
    );
}

#[test]
fn rust_flags_error_arms_that_answer_a_verdict_in_a_bool_result_function() {
    let source = "fn merged(root: &Path) -> Result<bool> {
    let tip = match resolve(root) {
        Ok(tip) => tip,
        Err(_) => return Ok(false),
    };
    match check(&tip) {
        Ok(status) => Ok(status),
        Err(..) => Ok(true),
    }
}
fn io_verdict(root: &Path) -> io::Result<bool> {
    if let Err(_) = probe(root) {
        return Ok(false);
    }
    match probe(root) {
        // DECISION: D001
        Err(_) => Ok(false),
        Ok(()) => Ok(true),
    }
}
";
    assert_eq!(
        silent_findings("source.rs", source),
        ["4:merged", "8:merged", "12:io_verdict"]
    );
}

#[test]
fn rust_keeps_propagating_guarded_aliased_deferred_and_test_error_arms() {
    let source = "fn propagated(root: &Path) -> Result<bool> {
    match probe(root) {
        Ok(()) => Ok(true),
        Err(error) if error.transient() => Ok(false),
        Err(error) => Err(error),
    }
}
type Flag = Result<bool>;
fn aliased(root: &Path) -> Flag {
    match probe(root) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}
fn deferred(root: &Path) -> Result<bool> {
    let inner = || -> Result<bool> {
        match probe(root) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    };
    inner()
}
#[test]
fn probe_test() -> Result<bool> {
    if let Err(_) = probe(Path::new(\".\")) {
        return Ok(false);
    }
    Ok(true)
}
";
    assert!(silent_findings("source.rs", source).is_empty());
}
