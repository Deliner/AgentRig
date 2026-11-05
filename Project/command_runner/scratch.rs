use super::jobs::{self, Job};
use crate::project_config::{Context, Lifetime};
use crate::shared::arguments::forwarded;
use crate::shared::console;
use crate::shared::fs;
use anyhow::{Result, ensure};
use std::{path::PathBuf, process::Command};

pub fn run(context: &Context, args: &[String]) -> Result<i32> {
    let text = forwarded(args).join(" ");
    ensure!(!text.trim().is_empty(), "scratch -- TEXT");
    let runtime = context.runtime.clone();
    let area = area(context, &runtime);
    fs::create_dir_all(area.join("results"))?;
    let argv = vec!["sh".to_owned(), "-c".to_owned(), text];
    let mut job = Job::create(&runtime, &context.root, "scratch", &context.root)?;
    job.prepare(&argv, jobs::Access::Writable, Lifetime::Task)?;
    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .current_dir(&context.root)
        .env("AGENTRIG_SCRATCH", &area);
    let result = jobs::process::execute(&mut command, jobs::Capture::Stream, Some(&mut job));
    if let Err(error) = &result {
        console::stderr(format!("{error:#}"));
    }
    let code = result.as_ref().map(jobs::process::exit_code).unwrap_or(127);
    job.finish(
        code,
        result.as_ref().err().map(|error| format!("{error:#}")),
    )?;
    Ok(code)
}
pub fn cleanup(context: &Context, branch: &str) -> Result<()> {
    let runtime = context.runtime.clone();
    let area = runtime.join("scratch").join(branch);
    let present = area.is_dir();
    if present {
        for entry in fs::read_dir(&area)? {
            let entry = entry?;
            let disposable = entry.file_name() != "results";
            match (disposable, entry.file_type()?.is_dir()) {
                (false, _) => {}
                // DECISION: D102
                (true, true) => fs::remove_dir_all(entry.path())?,
                (true, false) => fs::remove_file(entry.path())?,
            }
        }
    }
    for record in jobs::list(&runtime)? {
        let finished = record["finished"].is_u64();
        let owned = record["command"] == "scratch" && record["branch"] == branch;
        let run = record["run_id"].as_str().unwrap_or_default();
        let logged = owned && finished;
        if logged {
            // DECISION: D102
            fs::remove_dir_all(jobs::directory(&runtime, run)?)?;
        }
    }
    Ok(())
}
fn area(context: &Context, runtime: &std::path::Path) -> PathBuf {
    let branch = crate::shared::vcs::git_context(&context.root, &["branch", "--show-current"])
        .ok()
        .filter(|branch| !branch.is_empty())
        .unwrap_or_else(|| "detached".into());
    runtime.join("scratch").join(branch)
}
