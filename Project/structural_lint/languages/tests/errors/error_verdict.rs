use super::silent_findings;

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
    Ok(true)
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
