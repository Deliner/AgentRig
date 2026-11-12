use super::{
    WAIT, by_command, command, configured, jobs, parsed, pid, signal, spawn, status, until,
};
use crate::shared::testing::{configure, invoke, runtime, worker};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader},
    os::unix::process::ExitStatusExt,
    path::Path,
    process::Child,
    time::{Duration, Instant},
};

const IDENTITY: [(&str, &str); 2] = [
    ("WORKER_OWNER", "test-owner"),
    ("WORKER_PARENT_RUN", "parent-run"),
];
const STREAMS: &str =
    "head -c 131072 /dev/zero | LC_ALL=C tr '\\000' '\\377'; printf 'error stream' >&2";

fn supervised(root: &Path) -> (Child, Value) {
    let child = spawn(root, &["run", "wait"], &IDENTITY);
    let row = until("the running job", || {
        let rows = jobs(root);
        rows.into_iter().find(|row| row["state"] == "running")
    });
    (child, row)
}

#[test]
fn signal_reaches_the_foreground_child() {
    let script = "echo $$ > child.pid; exec sleep 60";
    let (_temp, root) = configured(&command("wait", &["sh", "-c", script]));
    let mut supervisor = spawn(&root, &["run", "wait"], &[]);
    let child = until("the child start", || {
        let text = fs::read_to_string(root.join("child.pid")).ok()?;
        text.trim().parse::<u64>().ok()
    });
    assert!(signal(supervisor.id().into(), "TERM"));
    assert_eq!(supervisor.wait().unwrap().code(), Some(143));
    assert!(!signal(child, "0"), "the child survived its supervisor");
}

// DECISION: D095
#[test]
fn foreground_drain_is_bounded_when_a_descendant_holds_the_pipe() {
    let leaky = command("leaky", &["sh", "-c", "sleep 30 & echo main-done"]);
    let (_temp, root) = configured(&leaky);
    let started = Instant::now();
    let result = invoke(&root, &["run", "leaky"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    assert!(started.elapsed() < Duration::from_secs(10));
}

// DECISION: D095
#[test]
fn worker_stays_signal_controllable_after_the_child_exits() {
    let ticks = "(for i in $(seq 200); do echo tick; sleep 0.05; done) & echo main-done";
    let (_temp, root) = configured(&command("chatty", &["sh", "-c", ticks]));
    let mut supervisor = spawn(&root, &["run", "chatty"], &[]);
    let output = BufReader::new(supervisor.stdout.take().unwrap());
    let mut lines = output.lines().map(|line| line.unwrap());
    let drained = lines
        .by_ref()
        .skip_while(|line| line != "main-done")
        .filter(|line| line == "tick")
        .take(3)
        .count();
    assert_eq!(drained, 3, "the worker did not reach the post-exit drain");
    assert!(signal(supervisor.id().into(), "TERM"));
    assert_eq!(supervisor.wait().unwrap().signal(), Some(15));
}

#[test]
fn live_command_identity_and_completion() {
    let (_temp, root) = configured(WAIT);
    let (mut supervisor, row) = supervised(&root);
    let identity = (&row["owner"], &row["parent_run"]);
    assert_eq!(identity, (&json!("test-owner"), &json!("parent-run")));
    assert_eq!(pid(&row["supervisor"]), u64::from(supervisor.id()));
    assert!(row["child"]["start_ticks"].as_u64().unwrap() > 0);
    assert!(row["leader_resources"]["resident_bytes"].as_u64().unwrap() > 0);
    assert!(row["branch"].is_null());
    assert!(signal(supervisor.id().into(), "TERM"));
    assert_eq!(supervisor.wait().unwrap().code(), Some(143));
    let finished = status(&root, row["run_id"].as_str().unwrap());
    let outcome = (&finished["state"], &finished["exit_code"]);
    assert_eq!(outcome, (&json!("completed"), &json!(143)));
    assert!(finished["leader_resources"].is_null());
}

#[test]
fn recovery_does_not_trust_a_saved_pid() {
    let (_temp, root) = configured(WAIT);
    let (mut supervisor, row) = supervised(&root);
    let identifier = row["run_id"].as_str().unwrap();
    let child = pid(&row["child"]);
    supervisor.kill().unwrap();
    supervisor.wait().unwrap();
    assert_eq!(status(&root, identifier)["state"], "orphaned");
    let path = root
        .join(".runtime/jobs")
        .join(identifier)
        .join("record.json");
    let mut saved: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let ticks = saved["child"]["start_ticks"].as_u64().unwrap();
    saved["child"]["start_ticks"] = json!(ticks + 1);
    fs::write(&path, saved.to_string()).unwrap();
    let stale = status(&root, identifier);
    assert_eq!(stale["state"], "interrupted");
    assert!(stale["leader_resources"].is_null());
    assert!(signal(child, "0"), "the recorded child stopped");
    assert!(signal(child, "KILL"));
}

#[test]
fn spawn_failure_is_recorded() {
    let missing = command("missing", &["/missing-worker-test-binary"]);
    let (_temp, root) = configured(&missing);
    assert_eq!(invoke(&root, &["run", "missing"]).code, Some(127));
    let row = &jobs(&root)[0];
    let outcome = (&row["state"], &row["exit_code"]);
    assert_eq!(outcome, (&json!("completed"), &json!(127)));
    assert!(row["error"].as_str().unwrap().contains("cannot execute"));
    assert_eq!(invoke(&root, &["job-status", "../escape"]).code, Some(2));
}

#[test]
fn job_list_skips_entries_without_a_record() {
    let (_temp, root) = configured("");
    assert_eq!(jobs(&root), Vec::<Value>::new());
    let directory = root.join(".runtime/jobs");
    fs::create_dir_all(directory.join("run-pending")).unwrap();
    fs::write(directory.join("stray"), "").unwrap();
    assert_eq!(invoke(&root, &["run", "fail"]).code, Some(23));
    let codes: Vec<_> = jobs(&root)
        .iter()
        .map(|row| row["exit_code"].clone())
        .collect();
    assert_eq!(codes, [json!(23)]);
}

#[test]
fn job_list_stops_on_an_unreadable_directory() {
    let (_temp, root) = configured("");
    let directory = root.join(".runtime/jobs");
    fs::create_dir_all(root.join(".runtime")).unwrap();
    std::os::unix::fs::symlink("jobs", &directory).unwrap();
    let listed = invoke(&root, &["jobs"]);
    assert_eq!(listed.code, Some(2), "{}", listed.output());
    let named = format!("read directory {}", directory.display());
    assert!(listed.stderr.contains(&named), "{}", listed.stderr);
}

#[test]
fn nested_command_inherits_owner_and_parent() {
    let (_temp, root) = configured("");
    let worker = worker().to_str().unwrap();
    let nested = [worker, "run", "--root", root.to_str().unwrap(), "fail"];
    configure(&root, "commands", &command("nested", &nested));
    let identity = [
        ("WORKER_OWNER", "nested-owner"),
        ("WORKER_PARENT_RUN", "external-parent"),
    ];
    let mut parent = spawn(&root, &["run", "nested"], &identity);
    assert_eq!(parent.wait().unwrap().code(), Some(23));
    let records = by_command(&root);
    let (parent, child) = (&records["nested"], &records["fail"]);
    assert_eq!(
        (&parent["owner"], &child["owner"]),
        (&json!("nested-owner"), &json!("nested-owner"))
    );
    assert_eq!(parent["parent_run"], "external-parent");
    assert_eq!(child["parent_run"], parent["run_id"]);
    let states = (&parent["state"], &child["state"]);
    assert_eq!(states, (&json!("completed"), &json!("completed")));
}

#[test]
fn logs_preserve_streams_and_bound_display() {
    let (_temp, root) = configured(&command("output", &["sh", "-c", STREAMS]));
    let result = runtime(&root, &["run", "output"]).output().unwrap();
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(result.stdout, vec![255; 131_072]);
    assert_eq!(result.stderr, b"error stream");
    let row = &jobs(&root)[0];
    assert!(row["duration_seconds"].as_f64().unwrap() > 0.0);
    let identifier = row["run_id"].as_str().unwrap();
    let logs = parsed(&invoke(&root, &["job-logs", identifier]));
    let stdout = &logs["stdout"];
    assert_eq!(
        (&stdout["truncated"], &stdout["bytes"]),
        (&json!(true), &json!(131_072))
    );
    assert_eq!(stdout["text"].as_str().unwrap().chars().count(), 65_536);
    assert_eq!(logs["stderr"]["text"], "error stream");
    let directory = root.join(".runtime/jobs").join(identifier);
    assert_eq!(
        fs::read(directory.join("stdout.log")).unwrap(),
        result.stdout
    );
    assert_eq!(
        fs::read(directory.join("stderr.log")).unwrap(),
        result.stderr
    );
}

#[test]
fn private_runner_cannot_restart_an_existing_job() {
    let (_temp, root) = configured("");
    assert_eq!(invoke(&root, &["run", "fail"]).code, Some(23));
    let identifier = jobs(&root)[0]["run_id"].as_str().unwrap().to_owned();
    let restarted = invoke(&root, &["_job-run", &identifier]);
    assert_eq!(restarted.code, Some(2), "{}", restarted.output());
    assert!(
        restarted.stderr.contains("already finished"),
        "{}",
        restarted.stderr
    );
}
