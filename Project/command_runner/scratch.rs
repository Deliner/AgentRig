use super::jobs::{self, Job};
use crate::project_config::{Context, Lifetime};
use crate::shared::arguments::forwarded;
use anyhow::{Result, ensure};
use std::{fs, path::PathBuf, process::Command};

pub fn run(context: &Context, args: &[String]) -> Result<i32> {
    let text = forwarded(args).join(" ");
    ensure!(!text.trim().is_empty(), "scratch -- TEXT");
    let runtime = context.path(&context.config.paths.runtime)?;
    let area = area(context, &runtime);
    fs::create_dir_all(area.join("results"))?;
    let argv = vec!["sh".to_owned(), "-c".to_owned(), text];
    let mut job = Job::create(&runtime, &context.root, "scratch", &context.root)?;
    job.prepare(&argv, false, Lifetime::Task)?;
    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .current_dir(&context.root)
        .env("AGENTRIG_SCRATCH", &area);
    let result = jobs::process::execute(&mut command, false, Some(&mut job));
    if let Err(error) = &result {
        eprintln!("{error:#}");
    }
    let code = result.as_ref().map(jobs::process::exit_code).unwrap_or(127);
    job.finish(
        code,
        result.as_ref().err().map(|error| format!("{error:#}")),
    )?;
    Ok(code)
}
fn area(context: &Context, runtime: &std::path::Path) -> PathBuf {
    let branch = crate::shared::vcs::git_context(&context.root, &["branch", "--show-current"])
        .ok()
        .filter(|branch| !branch.is_empty())
        .unwrap_or_else(|| "detached".into());
    runtime.join("scratch").join(branch)
}
