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

// DECISION: D126
#[test]
fn rust_flags_success_verdicts_of_process_statuses_without_code() {
    let source = "fn ancestor(root: &Path) -> Result<bool> {
    Ok(Command::new(\"git\")
        .arg(\"merge-base\")
        .status()?
        .success())
}
fn bound(root: &Path) -> Result<bool> {
    let status = git(root).status()?;
    if quiet(root) {
        return Ok(status.success());
    }
    Ok(status.success())
}
fn captured(root: &Path) -> anyhow::Result<bool> {
    let output = git(root).output()?;
    Ok(output.status.success())
}
fn chained(root: &Path) -> Result<bool> {
    Ok(git(root).output()?.status.success())
}
";
    assert_eq!(
        silent_findings("source.rs", source),
        [
            "2:success",
            "10:success",
            "12:success",
            "16:success",
            "19:success"
        ]
    );
}

// DECISION: D126
#[test]
fn rust_keeps_success_checks_that_read_the_code_or_answer_no_verdict() {
    let source = "fn coded(root: &Path) -> Result<bool> {
    let status = git(root).status()?;
    ensure!(matches!(status.code(), Some(0 | 1)), \"cannot inspect\");
    Ok(status.success())
}
fn read_code(root: &Path) -> Result<bool> {
    let output = git(root).output()?;
    let code = output.status.code();
    Ok(output.status.success())
}
fn plain(root: &Path) -> bool {
    git(root).status().is_ok_and(|status| status.success())
}
fn guarded(root: &Path) -> Result<()> {
    ensure!(git(root).status()?.success(), \"git failed\");
    Ok(())
}
";
    assert!(silent_findings("source.rs", source).is_empty());
}

// DECISION: D126
#[test]
fn rust_keeps_unlaunched_argued_deferred_justified_and_test_success_checks() {
    let source = "fn unlaunched(flag: &Flag) -> Result<bool> {
    Ok(flag.success())
}
fn argued(root: &Path) -> Result<bool> {
    Ok(git(root).status(1)?.success())
}
fn deferred(root: &Path) -> Result<bool> {
    let check = || -> Result<bool> { Ok(git(root).status()?.success()) };
    check()
}
fn justified(root: &Path) -> Result<bool> {
    // DECISION: D001
    Ok(git(root).status()?.success())
}
#[test]
fn probe() -> Result<bool> {
    Ok(git(Path::new(\".\")).status()?.success())
}
";
    assert!(silent_findings("source.rs", source).is_empty());
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
