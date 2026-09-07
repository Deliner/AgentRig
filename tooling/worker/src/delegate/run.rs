mod execution;
mod report;

use super::{
    config,
    sandbox::{self, Layout},
    task::{self, Request},
};
use crate::{
    jobs::{self, Job, Lifetime, Limits},
    util::save_json,
};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{env, fs, path::Path};

pub fn cli(root: &Path, runtime: &Path, args: &[String]) -> Result<i32> {
    let value = match args {
        [command, config] if command == "mcp" => {
            return super::mcp::serve(root, runtime, &root.join(config)).map(|()| 0);
        }
        [command, config, request] if command == "start" => {
            let request = serde_json::from_slice(&fs::read(root.join(request))?)?;
            start(root, runtime, &root.join(config), request)?
        }
        [command, id] if matches!(command.as_str(), "result" | "status") => result(runtime, id)?,
        [command, id] if command == "cancel" => cancel(runtime, id)?,
        _ => anyhow::bail!(
            "delegate mcp CONFIG | start CONFIG REQUEST | status RUN_ID | result RUN_ID | cancel RUN_ID"
        ),
    };
    println!("{value}");
    Ok(0)
}
pub fn cancel(runtime: &Path, id: &str) -> Result<Value> {
    result(runtime, id)?;
    jobs::cancel(runtime, id)?;
    result(runtime, id)
}

pub fn start(root: &Path, runtime: &Path, config_path: &Path, request: Request) -> Result<Value> {
    let config = config::load(config_path)?;
    let profile = config
        .profiles
        .get(&request.profile)
        .context("unknown delegate profile")?;
    task::validate(&request, profile)?;
    let mut job = Job::create(
        runtime,
        root,
        &format!("delegate:{}", request.profile),
        root,
    )?;
    let id = job.record().run_id.clone();
    let prepared = prepare(&mut job, &config, &request, runtime);
    if let Err(error) = prepared {
        report::finish(job.directory(), Err(error))?;
        job.finish(
            127,
            Some("delegate preparation failed; see delegate result".into()),
        )?;
        return Ok(json!({"run_id":id}));
    }
    let directory = job.directory().to_path_buf();
    if let Err(error) = job.launch(&env::current_exe()?) {
        report::finish(&directory, Err(error))?;
    }
    Ok(json!({"run_id":id}))
}

fn prepare(
    job: &mut Job,
    config: &config::Config,
    request: &Request,
    runtime: &Path,
) -> Result<()> {
    let profile = &config.profiles[&request.profile];
    let directory = job.directory();
    let input = directory.join("input");
    let inputs = task::prepare(
        (&job.record().project, &config.vcs),
        &input,
        request,
        profile,
    )?;
    sandbox::write_prompt(&input, profile)?;
    save_json(&directory.join("profile.json"), profile)?;
    save_json(&directory.join("request.json"), request)?;
    save_json(&directory.join("inputs.json"), &inputs)?;
    let codex = review_runner::execution::sandbox::native_codex_from("DELEGATE_CODEX_BIN")?;
    let layout = Layout {
        input,
        private: directory.join("private"),
        codex,
    };
    let environment = sandbox::prepare(&layout, profile)?;
    save_json(&directory.join("environment.json"), &environment)?;
    save_json(&directory.join("codex.json"), &layout.codex)?;
    prepare_job(job, runtime, profile)
}

fn prepare_job(job: &mut Job, runtime: &Path, profile: &config::Profile) -> Result<()> {
    let argv = vec![
        env::current_exe()?.to_string_lossy().into_owned(),
        "delegate".into(),
        "--root".into(),
        job.record().project.to_string_lossy().into_owned(),
        "_execute".into(),
        runtime.to_string_lossy().into_owned(),
        job.record().run_id.clone(),
    ];
    job.prepare(&argv, false, Lifetime::Task)?;
    job.limits(Limits {
        memory_bytes: profile.memory_bytes,
        max_processes: profile.max_processes,
    })
}

pub fn execute(runtime: &Path, id: &str) -> Result<i32> {
    let directory = jobs::directory(runtime, id)?;
    let status = jobs::status(runtime, id)?;
    let unit = status["scope"]["unit"]
        .as_str()
        .context("delegate needs scope containment")?;
    ensure!(
        fs::read_to_string("/proc/self/cgroup")?
            .trim()
            .ends_with(&format!("/{unit}")),
        "executor is outside assigned scope"
    );
    ensure!(
        !directory.join("delegate-result.json").exists(),
        "delegate already has a result"
    );
    let result = execution::run(&directory);
    let success = report::finish(&directory, result)?;
    Ok(i32::from(!success))
}

pub fn result(runtime: &Path, id: &str) -> Result<Value> {
    let directory = jobs::directory(runtime, id)?;
    let status = jobs::status(runtime, id)?;
    ensure!(
        status["command"]
            .as_str()
            .is_some_and(|name| name.starts_with("delegate:")),
        "run is not a delegate"
    );
    let terminal = matches!(
        status["state"].as_str(),
        Some("completed" | "cancelled" | "interrupted")
    );
    if terminal {
        report::recover(&directory, &status["state"])?;
    }
    let report = fs::read(directory.join("delegate-result.json"))
        .ok()
        .map(|bytes| serde_json::from_slice::<Value>(&bytes))
        .transpose()?;
    let outcome = outcome(&status, report.as_ref());
    let code = fs::read(directory.join("code-report.json"))
        .ok()
        .map(|bytes| serde_json::from_slice::<Value>(&bytes))
        .transpose()?;
    let environment = fs::read(directory.join("environment.json"))
        .ok()
        .map(|bytes| serde_json::from_slice::<Value>(&bytes))
        .transpose()?;
    Ok(
        json!({"run_id":id,"outcome":outcome,"job":status,"report":report,"code":code,"environment":environment}),
    )
}
fn outcome(status: &Value, report: Option<&Value>) -> &'static str {
    let passed = status["exit_code"] == 0
        && report.is_some_and(|report| {
            report["status"] == "PASS"
                && report["cleanup_errors"]
                    .as_array()
                    .is_some_and(Vec::is_empty)
        });
    match status["state"].as_str() {
        Some("completed") if passed => "PASS",
        Some("completed" | "interrupted") => "ERROR",
        Some("cancelled") => "CANCELLED",
        Some("unverified") => "UNKNOWN",
        _ => "RUNNING",
    }
}
