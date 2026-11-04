use crate::project_config::Lifetime;
use crate::shared::fs::{self, File, OpenOptions};
mod background;
pub mod cleanup;
mod group;
mod identity;
mod logs;
pub mod process;
pub mod report;
mod stop;
pub use stop::run as cancel;
mod storage;
mod streams;

use anyhow::Result;
use identity::Identity;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

// DECISION: D062
#[derive(Clone, Copy)]
pub enum Access {
    ReadOnly,
    Writable,
}
impl Access {
    fn reads(self) -> bool {
        matches!(self, Self::ReadOnly)
    }
}
// DECISION: D062
#[derive(Clone, Copy)]
pub enum Grouping {
    Own,
    Inherited,
}
impl Grouping {
    fn own(self) -> bool {
        matches!(self, Self::Own)
    }
    pub fn target(self, pid: i32) -> i32 {
        match self {
            Self::Own => -pid,
            Self::Inherited => pid,
        }
    }
}
// DECISION: D062
#[derive(Clone, Copy)]
pub enum Capture {
    Collect,
    Stream,
}
impl Capture {
    fn collects(self) -> bool {
        matches!(self, Self::Collect)
    }
}

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
    #[serde(default)]
    pub duration_seconds: Option<f64>,
    #[serde(default)]
    pub group: Option<Identity>,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub lifetime: Lifetime,
    #[serde(default)]
    pub background: bool,
}
pub struct Job {
    directory: PathBuf,
    record: Record,
    started: std::time::Instant,
}
impl Job {
    pub fn create(runtime: &Path, project: &Path, command: &str, cwd: &Path) -> Result<Self> {
        let directory = storage::create(runtime)?;
        let run_id = directory
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("job directory has no name"))?
            .to_string_lossy()
            .into_owned();
        let owner = owner().unwrap_or_else(|| run_id.clone());
        let branch = crate::shared::vcs::git_context(project, &["branch", "--show-current"])
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
            duration_seconds: None,
            group: None,
            read_only: false,
            lifetime: Lifetime::Task,
            background: false,
        };
        Self::registered(directory, record)
    }
    fn registered(directory: PathBuf, record: Record) -> Result<Self> {
        let job = Self {
            directory,
            record,
            started: std::time::Instant::now(),
        };
        job.save()?;
        Ok(job)
    }
    pub fn prepare(&mut self, argv: &[String], access: Access, lifetime: Lifetime) -> Result<()> {
        self.record.argv = argv.into();
        self.record.read_only = access.reads();
        self.record.lifetime = lifetime;
        self.save()
    }
    pub fn directory(&self) -> &Path {
        &self.directory
    }
    pub fn attach(&mut self, pid: u32, grouping: Grouping) -> Result<()> {
        self.record.child = Identity::read(pid).ok();
        self.record.process_group = grouping.own();
        self.save()
    }
    pub fn finish(&mut self, code: i32, error: Option<String>) -> Result<()> {
        self.record.duration_seconds = Some(self.started.elapsed().as_secs_f64());
        self.record.finished = Some(now()?);
        self.record.exit_code = Some(code);
        self.record.error = error;
        self.save()
    }
    fn save(&self) -> Result<()> {
        storage::save(&self.directory, &self.record)
    }
    fn open_logs(&self) -> Result<(File, File)> {
        let open = |name: &str| {
            fs::open_with(
                OpenOptions::new().create_new(true).write(true),
                self.directory.join(name),
            )
        };
        Ok((open("stdout.log")?, open("stderr.log")?))
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
    storage::list(runtime)?
        .iter()
        .map(|record| observe(runtime, record))
        .collect()
}
pub fn status(runtime: &Path, id: &str) -> Result<Value> {
    observe(runtime, &storage::load(runtime, id)?)
}
pub fn directory(runtime: &Path, id: &str) -> Result<PathBuf> {
    storage::load(runtime, id)?;
    Ok(runtime.join("jobs").join(id))
}
pub fn cli(runtime: &Path, command: &str, args: &[String]) -> Result<i32> {
    let cleanup = command == "job-cleanup";
    if cleanup {
        return cleanup_cli(runtime, args);
    }
    let value = match command {
        "jobs" => {
            anyhow::ensure!(args.is_empty(), "jobs takes no arguments");
            json!(list(runtime)?)
        }
        "job-status" => {
            anyhow::ensure!(args.len() == 1, "job-status RUN_ID");
            status(runtime, &args[0])?
        }
        "job-logs" => {
            anyhow::ensure!(args.len() == 1, "job-logs RUN_ID");
            logs::read(runtime, &args[0])?
        }
        "job-stop" => {
            anyhow::ensure!(args.len() == 1, "job-stop RUN_ID");
            stop::run(runtime, &args[0])?;
            status(runtime, &args[0])?
        }
        _ => anyhow::bail!("unknown jobs command {command}"),
    };
    println!("{value}");
    Ok(0)
}
fn cleanup_cli(runtime: &Path, args: &[String]) -> Result<i32> {
    let branch = match args {
        [] => None,
        [flag, branch] if flag == "--branch" => Some(branch.as_str()),
        _ => anyhow::bail!("job-cleanup [--branch BRANCH]"),
    };
    let result = cleanup::run(runtime, branch)?;
    let failed = result["errors"]
        .as_array()
        .is_some_and(|errors| !errors.is_empty());
    println!("{result}");
    Ok(i32::from(failed))
}
fn observe(runtime: &Path, record: &Record) -> Result<Value> {
    let value = observe_once(runtime, record)?;
    let interrupted = value["state"] == "interrupted";
    if interrupted {
        let latest = storage::load(runtime, &record.run_id)?;
        return observe_once(runtime, &latest);
    }
    Ok(value)
}
fn observe_once(runtime: &Path, record: &Record) -> Result<Value> {
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
    let finished = match record.finished {
        Some(seconds) => seconds,
        None => now()?,
    };
    value["age_seconds"] = json!(finished.saturating_sub(record.started));
    value["leader_resources"] = json!(child);
    value["containment"] = json!("process-group; detached descendants are not contained");
    let contained = record.background;
    if contained {
        return grouped(runtime, record, value);
    }
    Ok(value)
}
fn grouped(runtime: &Path, record: &Record, mut value: Value) -> Result<Value> {
    let directory = runtime.join("jobs").join(&record.run_id);
    let populated = record.group.as_ref().is_some_and(group::populated);
    let launching = record.group.is_none() && background::launcher_live(&directory)?;
    let stopped = directory.join("stop-requested").exists();
    value["state"] = json!(match (populated, stopped, record.finished) {
        (false, true, None) if launching => "stopping",
        (false, false, None) if launching => "starting-or-finishing",
        (true, true, _) => "stopping",
        (true, false, _) => "running",
        (false, true, _) => "cancelled",
        (false, false, Some(_)) => "completed",
        (false, false, None) if record.supervisor.observe().is_some() => "starting-or-finishing",
        _ => "interrupted",
    });
    value["containment"] = json!("process group");
    value["group_observation"] = json!(record.group.as_ref().map(group::observe));
    Ok(value)
}
fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
