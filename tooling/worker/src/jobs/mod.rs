mod identity;
mod storage;

use anyhow::Result;
use identity::Identity;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Deserialize, Serialize)]
pub struct Record {
    pub run_id: String,
    pub owner: String,
    pub parent_run: Option<String>,
    pub project: PathBuf,
    pub cwd: PathBuf,
    pub branch: Option<String>,
    pub command: String,
    pub argv: Vec<String>,
    pub started: u64,
    pub finished: Option<u64>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub supervisor: Identity,
    pub child: Option<Identity>,
    pub process_group: bool,
}
pub struct Job {
    directory: PathBuf,
    record: Record,
}
impl Job {
    pub fn create(runtime: &Path, project: &Path, command: &str, cwd: &Path) -> Result<Self> {
        let directory = storage::create(runtime)?;
        let run_id = directory
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let owner = owner().unwrap_or_else(|| run_id.clone());
        let branch = crate::util::git(project, &["branch", "--show-current"])
            .ok()
            .filter(|value| !value.is_empty());
        let record = Record {
            run_id,
            owner,
            parent_run: std::env::var("WORKER_PARENT_RUN").ok(),
            project: project.into(),
            cwd: cwd.into(),
            branch,
            command: command.into(),
            argv: Vec::new(),
            started: now()?,
            finished: None,
            exit_code: None,
            error: None,
            supervisor: Identity::read(std::process::id())?,
            child: None,
            process_group: false,
        };
        let job = Self { directory, record };
        job.save()?;
        Ok(job)
    }
    pub fn prepare(&mut self, argv: &[String]) -> Result<()> {
        self.record.argv = argv.into();
        self.save()
    }
    pub fn attach(&mut self, pid: u32, group: bool) -> Result<()> {
        self.record.child = Identity::read(pid).ok();
        self.record.process_group = group;
        self.save()
    }
    pub fn finish(&mut self, code: i32, error: Option<String>) -> Result<()> {
        self.record.finished = Some(now()?);
        self.record.exit_code = Some(code);
        self.record.error = error;
        self.save()
    }
    fn save(&self) -> Result<()> {
        storage::save(&self.directory, &self.record)
    }
}
pub fn owner() -> Option<String> {
    ["WORKER_OWNER", "CODEX_THREAD_ID", "CODEX_SESSION_ID"]
        .iter()
        .find_map(|key| {
            std::env::var(key)
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
}
pub fn list(runtime: &Path) -> Result<Vec<Value>> {
    storage::list(runtime)?.iter().map(observe).collect()
}
pub fn status(runtime: &Path, id: &str) -> Result<Value> {
    observe(&storage::load(runtime, id)?)
}
pub fn cli(runtime: &Path, command: &str, args: &[String]) -> Result<i32> {
    let value = match command {
        "jobs" => {
            anyhow::ensure!(args.is_empty(), "jobs takes no arguments");
            json!(list(runtime)?)
        }
        "job-status" => {
            anyhow::ensure!(args.len() == 1, "job-status RUN_ID");
            status(runtime, &args[0])?
        }
        _ => anyhow::bail!("unknown jobs command {command}"),
    };
    println!("{value}");
    Ok(0)
}
fn observe(record: &Record) -> Result<Value> {
    let child = record.child.as_ref().and_then(Identity::observe);
    let supervisor = record.supervisor.observe().is_some();
    let state = match (record.finished, child.is_some(), supervisor) {
        (Some(_), _, _) => "completed",
        (None, true, true) => "running",
        (None, true, false) => "orphaned",
        (None, false, true) => "starting-or-finishing",
        (None, false, false) => "interrupted",
    };
    let mut value = serde_json::to_value(record)?;
    value["state"] = json!(state);
    value["age_seconds"] = json!(
        record
            .finished
            .unwrap_or(now()?)
            .saturating_sub(record.started)
    );
    value["leader_resources"] = json!(child);
    value["containment"] = json!("process-group; detached descendants are not contained");
    Ok(value)
}
fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
