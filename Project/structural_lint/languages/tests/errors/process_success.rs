use super::silent_findings;

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
fn rust_keeps_unlaunched_argued_deferred_and_test_success_checks() {
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
#[test]
fn probe() -> Result<bool> {
    Ok(git(Path::new(\".\")).status()?.success())
}
";
    assert!(silent_findings("source.rs", source).is_empty());
}
