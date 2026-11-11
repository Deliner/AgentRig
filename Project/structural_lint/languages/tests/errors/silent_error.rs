use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use std::path::Path;

// DECISION: D097
fn silent_findings(path: &str, source: &str) -> Vec<String> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::SilentError))
        .collect();
    found.sort_by_key(|item| item.line);
    found
        .into_iter()
        .map(|item| format!("{}:{}", item.line, item.symbol))
        .collect()
}

// DECISION: D069
#[test]
fn python_flags_except_pass_without_a_justifying_reference() {
    let source = "def guarded():
    try:
        work()
    except ValueError:
        pass
    try:
        work()
    except KeyError:
        # DECISION: D001
        pass
    try:
        work()
    except OSError as error:
        log(error)
    try:
        work()
    except TypeError:
        cleanup()
        pass
    try:
        work()
    except IndexError: pass
";
    assert_eq!(
        silent_findings("source.py", source),
        ["4:except", "22:except"]
    );
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
fn rust_flags_filter_map_and_flat_map_that_keep_only_the_ok_values_of_any_chain() {
    let source =
        "fn load(items: &[&str], paths: &[PathBuf], text: &str, root: &Path) -> Result<()> {
    let parsed: Vec<u32> = items.iter().map(|item| item.parse()).filter_map(Result::ok).collect();
    let flat: Vec<u32> = items.iter().map(|item| item.parse()).flat_map(Result::ok).collect();
    let loaded: Vec<Config> = paths.iter().map(load).filter_map(|loaded| loaded.ok()).collect();
    let called: Vec<u32> = text.split(',').filter_map(|part| part.parse().ok()).collect();
    let numbers: Vec<u32> = text.lines().filter_map(|line| line.parse().ok()).collect();
    let pairs: Vec<u32> = table.iter().flat_map(|(_, value)| value.parse().ok()).collect();
    let entries: Vec<_> = fs::read_dir(root)?.flat_map(Result::ok).collect();
    Ok(())
}
";
    assert_eq!(
        silent_findings("source.rs", source),
        [
            "2:filter_map",
            "3:flat_map",
            "4:filter_map",
            "5:filter_map",
            "6:filter_map",
            "7:flat_map",
            "8:flat_map"
        ]
    );
}

#[test]
fn rust_leaves_lines_spellings_to_clippy_and_keeps_options_errors_justified_and_test_chains() {
    let source = "fn read(reader: impl BufRead, items: &[&str]) -> Vec<String> {
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
    let closure: Vec<String> = reader.lines().flat_map(|line| line.ok()).collect();
    let options: Vec<u32> = items.iter().filter_map(|item| lookup(item)).collect();
    let errors: Vec<Error> = results.into_iter().filter_map(Result::err).collect();
    let chained: Vec<u32> = items.iter().filter_map(|item| item.parse().ok().map(double)).collect();
    let flattened: Vec<u32> = items.iter().map(|item| item.parse()).flatten().collect();
    let bounded: Vec<u32> = items.iter().map(|item| item.parse()).map_while(Result::ok).collect();
    // DECISION: D001
    let justified: Vec<u32> = items.iter().filter_map(|item| item.parse().ok()).collect();
    lines
}
#[cfg(test)]
mod tests {
    fn probe(items: &[&str]) -> Vec<u32> {
        items.iter().filter_map(|item| item.parse().ok()).collect()
    }
}
";
    assert!(silent_findings("source.rs", source).is_empty());
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
