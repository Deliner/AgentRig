use super::{WAIT, background, by_command, command, configured, owned, parsed, pid, ready, status};
use crate::shared::testing::{configure, git, invoke, worker};
use serde_json::{Value, json};
use std::{fs, path::Path};

const DESCENDANT: &str = "sleep 60 & echo ready; sleep 60";
const STUBBORN: &str = "trap '' TERM; echo ready; sleep 60";
const FLAGGED: &str = "printf '%s\\n' \"$CARGO_ENCODED_RUSTFLAGS\"; echo ready; sleep 60";

fn stopped(root: &Path, owner: &str, identifier: &str) -> Value {
    parsed(&owned(root, owner, &["job-stop", identifier]))
}

fn emptied(state: &Value) {
    assert_eq!(state["state"], "cancelled", "{state}");
    assert_eq!(state["group_observation"]["populated"], false, "{state}");
}

#[test]
fn owner_cleanup_stops_a_nested_background_run() {
    let owner = "nested-group-owner";
    let (_temp, root) = configured(WAIT);
    let worker = worker().to_str().unwrap();
    let nested = [
        worker,
        "job-start",
        "--root",
        root.to_str().unwrap(),
        "wait",
    ];
    configure(&root, "commands", &command("nested", &nested));
    let started = parsed(&owned(&root, owner, &["run", "nested"]));
    let identifier = started["run_id"].as_str().unwrap();
    let records = by_command(&root);
    let (parent, child) = (&records["nested"], &records["wait"]);
    assert_eq!(child["parent_run"], parent["run_id"]);
    assert_eq!(
        (&parent["owner"], &child["owner"]),
        (&json!(owner), &json!(owner))
    );
    assert_eq!(pid(&child["group"]), pid(&child["supervisor"]));
    let cleanup = parsed(&owned(&root, owner, &["job-cleanup"]));
    let named = cleanup["stopped"]
        .as_array()
        .unwrap()
        .contains(&json!(identifier));
    assert!(named, "{cleanup}");
    emptied(&status(&root, identifier));
    stopped(&root, owner, identifier);
}

#[test]
fn background_ownership_and_descendant_cleanup() {
    let (_temp, root) = configured(&command("wait", &["sh", "-c", DESCENDANT]));
    let first = background(&root, "owner-one");
    let second = background(&root, "owner-two");
    ready(&root, &first);
    ready(&root, &second);
    let denied = owned(&root, "owner-two", &["job-stop", &first]);
    assert_eq!(denied.code, Some(2), "{}", denied.output());
    assert!(denied.stderr.contains("another owner"), "{}", denied.stderr);
    assert_eq!(status(&root, &first)["state"], "running");
    emptied(&stopped(&root, "owner-two", &second));
    assert_eq!(status(&root, &first)["state"], "running");
    stopped(&root, "owner-one", &first);
    stopped(&root, "owner-two", &second);
}

#[test]
fn forced_background_stop_keeps_logs() {
    let owner = "forced-stop-owner";
    let (_temp, root) = configured(&command("wait", &["sh", "-c", STUBBORN]));
    let identifier = background(&root, owner);
    ready(&root, &identifier);
    emptied(&stopped(&root, owner, &identifier));
    let logs = parsed(&invoke(&root, &["job-logs", &identifier]));
    assert_eq!(logs["stdout"]["text"], "ready\n");
    stopped(&root, owner, &identifier);
}

// DECISION: D182
#[test]
fn background_runs_compile_with_the_rig_flag_list() {
    let owner = "flag-list-owner";
    let (_temp, root) = configured(&command("wait", &["sh", "-c", FLAGGED]));
    let module = root.join(".agentrig/rust.yaml");
    fs::write(module, "rustflags: [--cfg, rig_flag]\n").unwrap();
    let identifier = background(&root, owner);
    ready(&root, &identifier);
    let logs = parsed(&invoke(&root, &["job-logs", &identifier]));
    assert_eq!(logs["stdout"]["text"], "--cfg\u{1f}rig_flag\nready\n");
    stopped(&root, owner, &identifier);
}

#[test]
fn cleanup_preserves_another_branch() {
    let owner = "branch-owner";
    let (_temp, root) = configured(WAIT);
    git(&root, &["init", "-q", "-b", "task/first"]);
    let first = background(&root, owner);
    git(&root, &["symbolic-ref", "HEAD", "refs/heads/task/second"]);
    let second = background(&root, owner);
    let cleanup = owned(&root, owner, &["job-cleanup", "--branch", "task/first"]);
    assert_eq!(cleanup.code, Some(0), "{}", cleanup.output());
    assert_eq!(status(&root, &first)["state"], "cancelled");
    assert_eq!(status(&root, &second)["state"], "running");
    stopped(&root, owner, &first);
    stopped(&root, owner, &second);
}
