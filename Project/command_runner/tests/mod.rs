mod background;
mod commands;
mod compiler_flags;
mod foreground;

use crate::shared::testing::{FAIL, Run, invoke, project, run, runtime};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;

const COMMANDS: &str = r#"echo:
  argv: [sh, -c, 'printf "%s\n" "$@"; cat; echo stderr >&2', echo]
  accepts_args: true
read:
  argv: []
  accepts_args: true
  programs: {cat: {}, sh: {}}
  read_only: true
"#;
const WAIT: &str = "wait:\n  argv: [sleep, '60']\n";

fn command(name: &str, argv: &[&str]) -> String {
    format!("{name}:\n  argv: {}\n", json!(argv))
}

fn configured(addition: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(&root, &format!("{FAIL}{COMMANDS}{addition}"), "[]\n");
    (temp, root)
}

fn owned(root: &Path, owner: &str, args: &[&str]) -> Run {
    run(runtime(root, args).env("WORKER_OWNER", owner), "")
}

fn parsed(result: &Run) -> Value {
    assert_eq!(result.code, Some(0), "{}", result.output());
    serde_json::from_str(&result.stdout).unwrap()
}

fn jobs(root: &Path) -> Vec<Value> {
    let rows = parsed(&invoke(root, &["jobs"]));
    rows.as_array().unwrap().clone()
}

fn by_command(root: &Path) -> BTreeMap<String, Value> {
    let rows = jobs(root).into_iter();
    rows.map(|row| (row["command"].as_str().unwrap().to_owned(), row))
        .collect()
}

fn pid(value: &Value) -> u64 {
    value["pid"].as_u64().unwrap()
}

fn status(root: &Path, identifier: &str) -> Value {
    parsed(&invoke(root, &["job-status", identifier]))
}

fn spawn(root: &Path, args: &[&str], environment: &[(&str, &str)]) -> Child {
    let mut command = runtime(root, args);
    command
        .envs(environment.iter().copied())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    command.spawn().unwrap()
}

fn signal(pid: u64, name: &str) -> bool {
    let status = Command::new("kill")
        .arg(format!("-{name}"))
        .arg(pid.to_string())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    status.success()
}

fn until<T>(event: &str, mut probe: impl FnMut() -> Option<T>) -> T {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Some(value) = probe() {
            return value;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("{event} did not happen");
}

fn background(root: &Path, owner: &str) -> String {
    let started = parsed(&owned(root, owner, &["job-start", "wait"]));
    started["run_id"].as_str().unwrap().to_owned()
}

fn ready(root: &Path, identifier: &str) {
    until("the readiness output", || {
        let logs = parsed(&invoke(root, &["job-logs", identifier]));
        let text = logs["stdout"]["text"].as_str().unwrap();
        text.contains("ready").then_some(())
    });
}
