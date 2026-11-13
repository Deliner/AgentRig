use super::resume;
use crate::project_config::Context;
use crate::shared::testing::{
    checked_repository, configure, git, hermetic, invoke, run, runtime, snapshot,
};
use serde_json::Value;
use std::{
    fs::{self, Permissions},
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::Path,
    process::Stdio,
    thread,
    time::{Duration, Instant},
};

const REPEATED: &str = "Repeated check failure";

fn checks(root: &Path) -> Value {
    resume(&Context::load(root).unwrap())
}

fn command(root: &Path, script: &str) {
    configure(
        root,
        "commands",
        &format!("fail:\n  argv: [sh, -c, '{script}']\n"),
    );
}

fn last_failures(checks: &Value) -> &Value {
    let results = checks["last_run"]["results"].as_array().unwrap();
    &results.last().unwrap()["consecutive_failures"]
}

fn started(path: &Path) -> u32 {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let written = fs::read_to_string(path).ok();
        if let Some(pid) = written.and_then(|text| text.trim().parse().ok()) {
            return pid;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("the check command did not start");
}

fn stop(group: u32) {
    let group = format!("-{group}");
    let result = run(hermetic("kill").args(["-s", "KILL", "--", &group]), "");
    assert_eq!(result.code, Some(0), "{}", result.output());
}

// INVARIANT: I018
#[test]
fn evidence_tracks_content_and_partial_checks() {
    let (_temp, root) = checked_repository();
    assert_eq!(checks(&root)["status"], "absent");
    let result = invoke(&root, &["check", "--only", "lint"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    let partial = checks(&root);
    assert_eq!(partial["current"], true, "{partial}");
    assert_eq!(partial["full_gate_passed"], false, "{partial}");
    assert_eq!(partial["last_run"]["only"], "lint", "{partial}");
    let source = root.join("src/value.py");
    fs::write(&source, "value = 2\n").unwrap();
    assert_eq!(checks(&root)["current"], false);
    fs::write(&source, "value = 1\n").unwrap();
    assert_eq!(checks(&root)["current"], true);
    fs::set_permissions(&source, Permissions::from_mode(0o755)).unwrap();
    assert_eq!(checks(&root)["current"], false);
}

#[test]
fn staged_evidence_does_not_verify_unstaged_source() {
    let (_temp, root) = checked_repository();
    let source = root.join("src/value.py");
    fs::write(&source, "line\n".repeat(61)).unwrap();
    let result = invoke(&root, &["check", "--staged", "--only", "lint"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    let staged = checks(&root);
    assert!(
        staged["last_run"]["source"].get("index").is_some(),
        "{staged}"
    );
    assert_eq!(staged["worktree_matches"], false, "{staged}");
    assert_eq!(staged["revision_matches"], true, "{staged}");
    git(&root, &["restore", "src/value.py"]);
    let before = snapshot(&root);
    assert_eq!(checks(&root)["current"], true);
    assert_eq!(snapshot(&root), before);
    fs::write(&source, "value = 2\n").unwrap();
    git(&root, &["add", "src/value.py"]);
    fs::write(&source, "value = 1\n").unwrap();
    let unstaged = checks(&root);
    assert_eq!(unstaged["worktree_matches"], true, "{unstaged}");
    assert_eq!(unstaged["index_matches"], false, "{unstaged}");
    assert_eq!(unstaged["current"], false, "{unstaged}");
}

#[test]
fn repeat_report_is_evidence_not_skill_read_claim() {
    let (_temp, root) = checked_repository();
    for _ in 0..2 {
        let result = invoke(&root, &["check"]);
        assert_eq!(result.code, Some(23), "{}", result.output());
    }
    assert!(invoke(&root, &["report"]).stdout.contains(REPEATED));
    let failed = checks(&root);
    assert_eq!(*last_failures(&failed), 2, "{failed}");
    assert_eq!(failed["full_gate_passed"], false, "{failed}");
    command(&root, "exit 0");
    let result = invoke(&root, &["check"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    let passed = checks(&root);
    assert_eq!(passed["full_gate_passed"], true, "{passed}");
    assert_eq!(*last_failures(&passed), 0, "{passed}");
    assert!(!invoke(&root, &["report"]).stdout.contains(REPEATED));
    git(&root, &["commit", "--allow-empty", "-qm", "new revision"]);
    let moved = checks(&root);
    assert_eq!(moved["worktree_matches"], true, "{moved}");
    assert_eq!(moved["revision_matches"], false, "{moved}");
    assert_eq!(moved["full_gate_passed"], false, "{moved}");
}

#[test]
fn changed_inputs_are_not_a_completed_proof() {
    let (_temp, root) = checked_repository();
    command(&root, "echo changed > src/value.py");
    let result = invoke(&root, &["check"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    let changed = checks(&root);
    assert_eq!(changed["last_run"]["status"], "inputs-changed", "{changed}");
    assert_eq!(changed["full_gate_passed"], false, "{changed}");
}

#[test]
fn interrupted_attempt_survives_fresh_resume() {
    let (_temp, root) = checked_repository();
    command(&root, "echo $$ > started; sleep 30");
    let mut check = runtime(&root, &["check"]);
    check.stdout(Stdio::null()).stderr(Stdio::null());
    let mut child = check.process_group(0).spawn().unwrap();
    let shell = started(&root.join("started"));
    stop(child.id());
    child.wait().unwrap();
    stop(shell);
    let interrupted = checks(&root);
    assert_eq!(
        interrupted["last_run"]["status"], "unfinished",
        "{interrupted}"
    );
    assert!(interrupted["last_run"]["code"].is_null(), "{interrupted}");
    assert_eq!(interrupted["full_gate_passed"], false, "{interrupted}");
}
