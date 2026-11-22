use crate::shared::console;
mod background;
mod cleanup;
mod group;
mod job;
mod logs;
mod pid_reuse_guard;
mod process;
mod record;
mod report;
mod stop;
mod storage;
mod streams;
pub use cleanup::run as clean_jobs;
pub use job::{Job, Launch, owner};
pub use process::{Capture, execute, exit_code, streamed};
pub use report::print as print_report;

use anyhow::Result;
use job::now;
use pid_reuse_guard::Identity;
use record::Record;
use serde::Serialize;
use serde_json::{Value, json};
use std::path::Path;

pub fn list(runtime: &Path) -> Result<Vec<Value>> {
    storage::list(runtime)?
        .iter()
        .map(|record| observe(runtime, record))
        .collect()
}
pub fn status(runtime: &Path, id: &str) -> Result<Value> {
    observe(runtime, &storage::load(runtime, id)?)
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Command {
    List,
    Status,
    Logs,
    Stop,
    Cleanup,
}
pub fn cli(runtime: &Path, command: Command, args: &[String]) -> Result<i32> {
    let value = match command {
        Command::Cleanup => return cleanup_cli(runtime, args),
        Command::List => {
            anyhow::ensure!(args.is_empty(), "jobs takes no arguments");
            json!(list(runtime)?)
        }
        Command::Status => {
            anyhow::ensure!(args.len() == 1, "job-status RUN_ID");
            status(runtime, &args[0])?
        }
        Command::Logs => {
            anyhow::ensure!(args.len() == 1, "job-logs RUN_ID");
            logs::read(runtime, &args[0])?
        }
        Command::Stop => {
            anyhow::ensure!(args.len() == 1, "job-stop RUN_ID");
            stop::run(runtime, &args[0])?;
            status(runtime, &args[0])?
        }
    };
    console::stdout(value);
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
    console::stdout(result);
    Ok(i32::from(failed))
}
#[derive(Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum JobState {
    Completed,
    Running,
    Orphaned,
    StartingOrFinishing,
    Interrupted,
    Stopping,
    Cancelled,
}
struct Observation {
    state: JobState,
    value: Value,
}
fn observe(runtime: &Path, record: &Record) -> Result<Value> {
    let observation = observe_once(runtime, record)?;
    match observation.state {
        JobState::Interrupted => {
            let latest = storage::load(runtime, &record.run_id)?;
            Ok(observe_once(runtime, &latest)?.value)
        }
        JobState::Completed
        | JobState::Running
        | JobState::Orphaned
        | JobState::StartingOrFinishing
        | JobState::Stopping
        | JobState::Cancelled => Ok(observation.value),
    }
}
fn observe_once(runtime: &Path, record: &Record) -> Result<Observation> {
    let child = record.child.as_ref().and_then(Identity::observe);
    let supervisor = record.supervisor.observe().is_some();
    let state = match (record.finished, child.is_some(), supervisor) {
        (Some(_), _, _) => JobState::Completed,
        (None, true, true) => JobState::Running,
        (None, true, false) => JobState::Orphaned,
        (None, false, true) => JobState::StartingOrFinishing,
        (None, false, false) => JobState::Interrupted,
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
    Ok(Observation { state, value })
}
fn grouped(runtime: &Path, record: &Record, mut value: Value) -> Result<Observation> {
    let directory = runtime.join("jobs").join(&record.run_id);
    let populated = record.group.as_ref().is_some_and(group::populated);
    let launching = record.group.is_none() && background::launcher_live(&directory)?;
    let stopped = directory.join("stop-requested").exists();
    let supervised = || record.supervisor.observe().is_some();
    let state = match (populated, stopped, record.finished) {
        (false, true, None) if launching => JobState::Stopping,
        (false, false, None) if launching => JobState::StartingOrFinishing,
        (true, true, _) => JobState::Stopping,
        (true, false, _) => JobState::Running,
        (false, true, _) => JobState::Cancelled,
        (false, false, Some(_)) => JobState::Completed,
        (false, false, None) if supervised() => JobState::StartingOrFinishing,
        _ => JobState::Interrupted,
    };
    value["state"] = json!(state);
    value["containment"] = json!("process group");
    value["group_observation"] = json!(record.group.as_ref().map(group::observe));
    Ok(Observation { state, value })
}
