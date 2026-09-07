use super::{Request, prepare, validate, verify};
use crate::delegate::config::{Mode, Profile};
use review_runner::snapshot;
use serde_json::json;
use std::{fs, path::Path};
use std::{os::unix::fs::symlink, process::Command};

fn profile() -> Profile {
    serde_json::from_value(json!({
        "frontend":"codex", "model":"configured", "reasoning_effort":"high",
        "mode":"artifacts", "prompt":"unused.md", "visible_paths":["src/**"],
        "timeout_seconds":60, "credentials":{"codex_auth_file_env":"AUTH_FILE"}
    }))
    .unwrap()
}
fn request() -> Request {
    serde_json::from_value(json!({
        "profile":"worker", "task":"Produce the required result",
        "contract":{"result_schema":{"type":"object","required":["ok"],
            "properties":{"ok":{"const":true}},"additionalProperties":false},
            "artifacts":{"output.bin":4}}
    }))
    .unwrap()
}
fn git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(root)
        .env("GIT_AUTHOR_NAME", "Worker Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
        .env("GIT_COMMITTER_NAME", "Worker Test")
        .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
        .args(args)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn snapshot_preserves_fixed_revision_and_checkout() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("src")).unwrap();
    let file = root.path().join("src/value.txt");
    fs::write(&file, "committed").unwrap();
    git(root.path(), &["init", "-q"]);
    git(root.path(), &["add", "src"]);
    git(root.path(), &["commit", "-qm", "input"]);
    let expected = snapshot::resolve(root.path(), "HEAD").unwrap();
    fs::write(&file, "uncommitted").unwrap();
    let mut request = request();
    request.revision = Some(expected.clone());
    let output = root.path().join("input");
    let inputs = prepare(
        (root.path(), &Default::default()),
        &output,
        &request,
        &profile(),
    )
    .unwrap();
    assert_eq!(inputs.revision, Some(expected));
    assert_eq!(
        fs::read_to_string(output.join("project/src/value.txt")).unwrap(),
        "committed"
    );
    assert_eq!(fs::read_to_string(file).unwrap(), "uncommitted");
    assert!(!output.join("project/.git").exists());
    assert!(
        prepare(
            (root.path(), &Default::default()),
            &output,
            &request,
            &profile()
        )
        .is_err()
    );
}

#[test]
fn explicit_inputs_work_without_git_and_enforce_visibility() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("src")).unwrap();
    fs::write(root.path().join("src/input.bin"), [0, 255]).unwrap();
    let mut request = request();
    request
        .inputs
        .insert("asset.bin".into(), "src/input.bin".into());
    let output = root.path().join("input");
    let inputs = prepare(
        (root.path(), &Default::default()),
        &output,
        &request,
        &profile(),
    )
    .unwrap();
    assert!(inputs.revision.is_none());
    assert_eq!(fs::read(output.join("inputs/asset.bin")).unwrap(), [0, 255]);
    request.inputs.insert("hidden".into(), "other/file".into());
    let error = prepare(
        (root.path(), &Default::default()),
        &root.path().join("denied"),
        &request,
        &profile(),
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("outside visible_paths"));
}

#[test]
fn contracts_reject_unverifiable_results_and_symlink_artifacts() {
    let root = tempfile::tempdir().unwrap();
    let request = request();
    fs::write(root.path().join("result.json"), r#"{"ok":true}"#).unwrap();
    fs::write(root.path().join("output.bin"), [1, 2]).unwrap();
    let result = verify(root.path(), &request.contract).unwrap();
    assert_eq!(result["artifacts"]["output.bin"]["bytes"], 2);
    fs::write(root.path().join("output.bin"), [1; 5]).unwrap();
    assert!(verify(root.path(), &request.contract).is_err());
    fs::remove_file(root.path().join("output.bin")).unwrap();
    symlink("result.json", root.path().join("output.bin")).unwrap();
    assert!(verify(root.path(), &request.contract).is_err());
    fs::remove_file(root.path().join("output.bin")).unwrap();
    fs::write(root.path().join("output.bin"), []).unwrap();
    fs::write(root.path().join("result.json"), r#"{"ok":false}"#).unwrap();
    assert!(verify(root.path(), &request.contract).is_err());
}

#[test]
fn request_rejects_traversal_and_artifacts_in_read_mode() {
    let mut request = request();
    request
        .inputs
        .insert("../escape".into(), "src/input".into());
    assert!(validate(&request, &profile()).is_err());
    request.inputs.clear();
    let mut profile = profile();
    profile.mode = Mode::Read;
    assert!(validate(&request, &profile).is_err());
    request.contract.artifacts.clear();
    assert!(validate(&request, &profile).is_ok());
}
