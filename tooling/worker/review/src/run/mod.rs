mod previous;
pub mod report;
use crate::{
    config::{self, Config},
    contract, digest,
    execution::{
        reviewer::{self, Task},
        sandbox::{self, Layout},
    },
    response::{self, Expected},
    snapshot,
};
use anyhow::{Context, Result};
use report::Report;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub root: PathBuf,
    pub base: String,
    pub candidate: String,
    pub previous_report: Option<PathBuf>,
    pub tool: String,
}
pub fn run(path: &Path, request: Request) -> Result<Report> {
    let path = path.canonicalize()?;
    let config = config::load(&path)?;
    let tool = config
        .tools
        .get(&request.tool)
        .context("unknown review tool")?;
    fs::create_dir_all(&config.runner.runtime_root)?;
    let directory = tempfile::Builder::new()
        .prefix("run-")
        .tempdir_in(&config.runner.runtime_root)?;
    let run_id = directory
        .path()
        .file_name()
        .context("run ID missing")?
        .to_string_lossy()
        .into_owned();
    let mut report = new_report(&config, &request, run_id)?;
    report
        .resource_digests
        .insert("configuration".into(), digest(&fs::read(&path)?));
    let deadline = Instant::now() + Duration::from_secs(config.runner.timeout_seconds);
    if let Err(error) = execute(&config, request, directory.path(), (&mut report, deadline)) {
        report.technical_error = Some(format!("{error:#}"));
    }
    report.aggregate(tool.reviewers.len());
    finish(directory, &mut report, &config.runner.report_root)?;
    Ok(report)
}
fn finish(directory: tempfile::TempDir, report: &mut Report, root: &Path) -> Result<()> {
    if let Err(error) = report.save(root) {
        report.technical_error = Some(format!("report persistence failed: {error:#}"));
        report.verdict = "BLOCKED".into();
        let _ = report.save(directory.path());
        let retained = directory.keep();
        return Err(error.context(format!(
            "report persistence failed; evidence retained at {}",
            retained.display()
        )));
    }
    let temporary = directory.path().to_owned();
    if let Err(error) = directory.close() {
        report.cleanup_error = Some(error.to_string());
    }
    let remains = temporary.exists();
    if remains {
        report.cleanup_error = Some(format!(
            "runtime directory remains: {}",
            temporary.display()
        ));
    }
    report.save(root)?;
    Ok(())
}
fn new_report(config: &Config, request: &Request, run_id: String) -> Result<Report> {
    Ok(Report {
        schema_version: 1,
        run_id,
        tool: request.tool.clone(),
        request: serde_json::to_value(request)?,
        snapshot: None,
        configuration: serde_json::to_value(config)?,
        project_configuration: serde_json::Value::Null,
        contract: serde_json::Value::Null,
        contract_digest: String::new(),
        response_schema: serde_json::from_str(response::SCHEMA)?,
        prompts: Default::default(),
        resource_digests: Default::default(),
        previous_report_digest: None,
        repair_diff: None,
        roles: vec![],
        verdict: "BLOCKED".into(),
        technical_error: None,
        cleanup_error: None,
    })
}
fn execute(
    config: &Config,
    mut request: Request,
    directory: &Path,
    progress: (&mut Report, Instant),
) -> Result<()> {
    let (report, deadline) = progress;
    request.root = request.root.canonicalize()?;
    report.request["root"] = serde_json::to_value(&request.root)?;
    let (contract, project) = resources(config, &request.tool, report)?;
    report.snapshot = Some(snapshot::prepare(
        &request.root,
        (&request.base, &request.candidate),
        &project.repository,
        &directory.join("project"),
    )?);
    let previous = previous::prepare(&request, report, &project.repository)?;
    inputs(directory, report, previous.as_ref())?;
    reviewers(config, directory, report, (&contract, deadline))?;
    previous::validate(report, previous.as_ref())?;
    Ok(())
}
fn reviewers(
    config: &Config,
    directory: &Path,
    report: &mut Report,
    inputs: (&contract::Contract, Instant),
) -> Result<()> {
    let (contract, deadline) = inputs;
    let codex = sandbox::native_codex()?;
    for roles in config.tools[&report.tool]
        .reviewers
        .chunks(config.runner.parallelism)
    {
        let outcomes = std::thread::scope(|scope| {
            let mut threads = Vec::new();
            for role in roles {
                let task = task(
                    config,
                    role,
                    (directory, &codex),
                    (report, contract, deadline),
                )?;
                threads.push(scope.spawn(move || reviewer::review(task)));
            }
            threads
                .into_iter()
                .map(|thread| {
                    thread
                        .join()
                        .map_err(|_| anyhow::anyhow!("reviewer thread panicked"))
                })
                .collect::<Result<Vec<_>>>()
        })?;
        report.roles.extend(outcomes);
    }
    Ok(())
}
fn resources(
    config: &Config,
    tool: &str,
    report: &mut Report,
) -> Result<(contract::Contract, config::Project)> {
    let tool = &config.tools[tool];
    let project = config::project(&tool.project_config)?;
    let bytes = fs::read(&project.review.contract)?;
    let contract: contract::Contract = serde_json::from_slice(&bytes)?;
    contract.validate(&tool.reviewers)?;
    report.contract_digest = digest(&bytes);
    report.contract = serde_json::from_slice(&bytes)?;
    report.project_configuration = serde_json::to_value(&project)?;
    report.resource_digests.insert(
        "project_configuration".into(),
        digest(&fs::read(&tool.project_config)?),
    );
    report.resource_digests.insert(
        "response_schema".into(),
        digest(response::SCHEMA.as_bytes()),
    );
    for role in &tool.reviewers {
        let prompt = fs::read_to_string(&config.reviewers[role].prompt)?;
        report
            .resource_digests
            .insert(format!("prompt:{role}"), digest(prompt.as_bytes()));
        report.prompts.insert(role.clone(), prompt);
    }
    Ok((contract, project))
}
fn task<'a>(
    config: &'a Config,
    role: &str,
    paths: (&Path, &Path),
    inputs: (&Report, &contract::Contract, Instant),
) -> Result<Task<'a>> {
    let (report, contract, deadline) = inputs;
    let snapshot = report.snapshot.as_ref().context("snapshot missing")?;
    let expected = Expected {
        run_id: report.run_id.clone(),
        candidate: snapshot.candidate.clone(),
        contract_digest: report.contract_digest.clone(),
        role: role.into(),
        contract: contract.clone(),
    };
    Ok(Task {
        layout: Layout {
            project: paths.0.join("project"),
            input: paths.0.join("input"),
            role: paths.0.join("reviewers").join(role),
            codex: paths.1.to_owned(),
        },
        expected,
        reviewer: &config.reviewers[role],
        deadline,
        attempts: config.runner.format_attempts,
        instructions: report.prompts[role].clone(),
    })
}
fn inputs(directory: &Path, report: &mut Report, previous: Option<&Report>) -> Result<()> {
    let input = directory.join("input");
    fs::create_dir(&input)?;
    fs::write(
        input.join("contract.json"),
        serde_json::to_vec_pretty(&report.contract)?,
    )?;
    fs::write(input.join("response-schema.json"), response::SCHEMA)?;
    fs::write(
        input.join("manifest.json"),
        serde_json::to_vec_pretty(&report.snapshot)?,
    )?;
    fs::write(
        input.join("diff.txt"),
        &report.snapshot.as_ref().context("snapshot missing")?.diff,
    )?;
    if let Some(previous) = previous {
        fs::write(
            input.join("previous.json"),
            serde_json::to_vec_pretty(previous)?,
        )?;
    }
    if let Some(diff) = &report.repair_diff {
        fs::write(input.join("repair-diff.txt"), diff)?;
    }
    Ok(())
}
