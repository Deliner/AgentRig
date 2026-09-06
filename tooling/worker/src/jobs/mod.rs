mod background;
pub mod cleanup;
mod identity;
mod logs;
pub mod process;
pub mod report;
mod scope;
pub use scope::capability;
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

#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Lifetime {
    #[default]
    Task,
    Shared,
}
#[derive(Default, Deserialize, Serialize)]
pub struct Limits {
    pub memory_bytes: Option<u64>,
    pub max_processes: Option<u64>,
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
    pub scope: Option<scope::Scope>,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub lifetime: Lifetime,
    #[serde(default)]
    pub background: bool,
    #[serde(default)]
    pub limits: Limits,
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
            duration_seconds: None,
            scope: None,
            read_only: false,
            lifetime: Lifetime::Task,
            background: false,
            limits: Limits::default(),
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
    pub fn prepare(&mut self, argv: &[String], read_only: bool, lifetime: Lifetime) -> Result<()> {
        self.record.argv = argv.into();
        self.record.read_only = read_only;
        self.record.lifetime = lifetime;
        self.save()
    }
    pub fn limits(&mut self, limits: Limits) -> Result<()> {
        self.record.limits = limits;
        self.save()
    }
    pub fn directory(&self) -> &Path {
        &self.directory
    }
    pub fn attach(&mut self, pid: u32, group: bool) -> Result<()> {
        self.record.child = Identity::read(pid).ok();
        self.record.process_group = group;
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
    fn open_logs(&self) -> Result<(std::fs::File, std::fs::File)> {
        use std::fs::OpenOptions;
        let open = |name: &str| {
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(self.directory.join(name))
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
    value["age_seconds"] = json!(
        record
            .finished
            .unwrap_or(now()?)
            .saturating_sub(record.started)
    );
    value["leader_resources"] = json!(child);
    value["containment"] = json!("process-group; detached descendants are not contained");
    scoped(runtime, record, value)
}
fn scoped(runtime: &Path, record: &Record, mut value: Value) -> Result<Value> {
    if let Some(scope) = &record.scope {
        let observed = match scope.observe() {
            Ok(observed) => observed,
            Err(error) => {
                value["state"] = json!("unverified");
                value["observation_error"] = json!(error.to_string());
                return Ok(value);
            }
        };
        let populated = observed["populated"] == true;
        let launching = scope.invocation.is_none()
            && background::launcher_live(&runtime.join("jobs").join(&record.run_id))?;
        let stopped = runtime
            .join("jobs")
            .join(&record.run_id)
            .join("stop-requested")
            .exists();
        value["state"] = json!(match (populated, stopped, record.finished) {
            (false, true, None) if launching => "stopping",
            (false, false, None) if launching => "starting-or-finishing",
            (true, true, _) => "stopping",
            (true, false, _) => "running",
            (false, true, _) => "cancelled",
            (false, false, Some(_)) => "completed",
            (false, false, None) if record.supervisor.observe().is_some() =>
                "starting-or-finishing",
            _ => "interrupted",
        });
        value["containment"] = json!("systemd user scope");
        value["scope_observation"] = observed;
    }
    Ok(value)
}
fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
