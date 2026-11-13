use super::{installed, merge, start};
use crate::shared::testing::{Run, commit, configure, hermetic, invoke, run, runtime, worker};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader},
    mem::DropGuard,
    path::Path,
    process::{Child, Stdio},
    time::{Duration, Instant},
};

const BACKGROUND: &str = r#"wait:
  purpose: Wait as a task job
  argv: [sh, -c, 'echo ready; exec sleep 60']
service:
  purpose: Wait as a service
  argv: [sh, -c, 'echo ready; exec sleep 60']
  lifetime: shared
"#;

fn owned(root: &Path, owner: &str, args: &[&str]) -> Run {
    run(runtime(root, args).env("WORKER_OWNER", owner), "")
}

fn json(result: &Run) -> Value {
    assert_eq!(result.code, Some(0), "{}", result.output());
    serde_json::from_str(&result.stdout).unwrap()
}

struct Jobs<'a> {
    root: &'a Path,
    started: Vec<(&'static str, String)>,
}

impl Jobs<'_> {
    fn start(&mut self, owner: &'static str, command: &str) -> String {
        let started = json(&owned(self.root, owner, &["job-start", command]));
        let identifier = started["run_id"].as_str().unwrap().to_owned();
        self.started.push((owner, identifier.clone()));
        identifier
    }

    fn ready(&self, identifier: &str) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            let logs = json(&invoke(self.root, &["job-logs", identifier]));
            let ready = logs["stdout"]["text"].as_str().unwrap().contains("ready");
            if ready {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("{identifier}: the background child did not report readiness");
    }

    fn state(&self, identifier: &str) -> Value {
        json(&owned(
            self.root,
            "merge-owner",
            &["job-status", identifier],
        ))["state"]
            .clone()
    }
}

fn stop(jobs: Jobs<'_>) {
    for (owner, identifier) in &jobs.started {
        owned(jobs.root, owner, &["job-stop", identifier]);
    }
}

fn kill(mut child: Child) {
    let process = child.id().to_string();
    hermetic("kill").arg(process).status().unwrap();
    child.wait().unwrap();
}

#[test]
fn merge_cleans_only_owned_task_runs() {
    let (_temp, root) = installed();
    start(&root, "jobs");
    configure(&root, "commands", BACKGROUND);
    commit(&root, "work.txt", "feature");
    let started = Jobs {
        root: &root,
        started: Vec::new(),
    };
    let mut jobs = DropGuard::new(started, stop);
    let task = jobs.start("merge-owner", "wait");
    let service = jobs.start("merge-owner", "service");
    let other = jobs.start("other-owner", "wait");
    for identifier in [&task, &service, &other] {
        jobs.ready(identifier);
    }
    let merged = owned(&root, "merge-owner", &["feature-merge"]);
    assert_eq!(merged.code, Some(0), "{}", merged.output());
    assert!(
        merged.stdout.contains("shared service"),
        "{}",
        merged.stdout
    );
    assert_eq!(jobs.state(&task), "cancelled");
    assert_eq!(jobs.state(&service), "running");
    assert_eq!(jobs.state(&other), "running");
}

#[test]
fn merge_inside_a_run_lists_only_other_live_runs() {
    let (_temp, root) = installed();
    start(&root, "inside");
    configure(&root, "commands", BACKGROUND);
    commit(&root, "work.txt", "feature");
    let mut waiting = hermetic(worker());
    waiting
        .args(["run", "--root"])
        .arg(&root)
        .arg("wait")
        .env("WORKER_OWNER", "merge-owner")
        .stdin(Stdio::null())
        .stdout(Stdio::piped());
    let mut foreign = DropGuard::new(waiting.spawn().unwrap(), kill);
    let mut line = String::new();
    let output = foreign.stdout.as_mut().unwrap();
    BufReader::new(output).read_line(&mut line).unwrap();
    assert_eq!(line, "ready\n");
    let rows = json(&owned(&root, "merge-owner", &["jobs"]));
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["command"] == "wait");
    let identifier = row.unwrap()["run_id"].clone();
    let text = format!(
        "{} feature-merge --root {}",
        worker().display(),
        root.display()
    );
    let merged = owned(&root, "merge-owner", &["scratch", "--", &text]);
    assert_eq!(merged.code, Some(0), "{}", merged.output());
    let (_, cleanup) = merged.stdout.split_once("post-merge cleanup: ").unwrap();
    let report: Value = serde_json::from_str(cleanup.lines().next().unwrap()).unwrap();
    let reason = "uncontained run; inspect manually";
    let retained = json!([{"run_id": identifier, "reason": reason}]);
    assert_eq!(report["retained"], retained);
    assert_eq!(report["stopped"], json!([]));
    assert_eq!(report["errors"], json!([]));
}

#[test]
fn merge_keeps_scratch_results_and_removes_temporary_runs() {
    let (_temp, root) = installed();
    start(&root, "probe");
    let text =
        r#"echo kept > "$AGENTRIG_SCRATCH/results/out" && echo draft > "$AGENTRIG_SCRATCH/draft""#;
    let scratch = invoke(&root, &["scratch", "--", text]);
    assert_eq!(scratch.code, Some(0), "{}", scratch.output());
    commit(&root, "work.txt", "feature");
    merge(&root);
    let area = root.join(".agentrig/runtime/scratch/task/probe");
    let names: Vec<_> = fs::read_dir(&area)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, ["results"]);
    assert_eq!(
        fs::read_to_string(area.join("results/out")).unwrap(),
        "kept\n"
    );
    let rows = json(&invoke(&root, &["jobs"]));
    let commands: Vec<_> = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|row| &row["command"])
        .collect();
    assert!(!commands.contains(&&json!("scratch")), "{commands:?}");
}
