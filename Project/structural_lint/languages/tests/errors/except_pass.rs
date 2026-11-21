use super::silent_findings;

// DECISION: D069
#[test]
fn python_flags_except_pass() {
    let source = "def guarded():
    try:
        work()
    except ValueError:
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
        ["4:except", "17:except"]
    );
}
