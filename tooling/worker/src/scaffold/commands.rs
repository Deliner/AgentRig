// DECISION: D005
use super::{config::Context, process};
use anyhow::{Context as _, Result, ensure};
use std::process::Output;

pub fn argv(context: &Context, name: &str, extra: &[String]) -> Result<Vec<String>> {
    let command = context
        .config
        .commands
        .get(name)
        .with_context(|| format!("unknown command {name}"))?;
    ensure!(
        extra.is_empty() || command.accepts_args,
        "command {name} does not accept arguments"
    );
    let argv: Vec<_> = command.argv.iter().chain(extra).cloned().collect();
    ensure!(!argv.is_empty(), "command {name} needs argv");
    Ok(argv)
}
pub fn run(context: &Context, name: &str, extra: &[String]) -> Result<i32> {
    let argv = argv(context, name, extra)?;
    match execute(context, name, argv, false) {
        Ok(output) => Ok(process::exit_code(&output)),
        Err(error) => {
            eprintln!("{error:#}");
            Ok(127)
        }
    }
}
pub fn execute(context: &Context, name: &str, argv: Vec<String>, capture: bool) -> Result<Output> {
    let spec = &context.config.commands[name];
    let cwd = context.path(&spec.cwd)?;
    let runtime = context.path(&context.config.paths.runtime)?;
    let mut job = discipline_worker::jobs::Job::create(&runtime, &context.root, name, &cwd)?;
    job.prepare(&argv, spec.read_only, spec.lifetime)?;
    let scoped = context.config.processes.foreground == super::config::Containment::Systemd;
    if scoped {
        return job.foreground(&std::env::current_exe()?, capture);
    }
    let result = process::tracked(&cwd, &argv, (spec.read_only, capture), Some(&mut job));
    let code = result.as_ref().map(process::exit_code).unwrap_or(127);
    job.finish(
        code,
        result.as_ref().err().map(|error| format!("{error:#}")),
    )?;
    result
}
pub fn report(context: &Context) -> Result<()> {
    discipline_worker::jobs::report::print(&context.path(&context.config.paths.runtime)?)
}
pub fn background(context: &Context, args: &[String]) -> Result<i32> {
    let (name, extra) = args
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("job-start COMMAND [-- ARGS]"))?;
    let extra = extra.strip_prefix(&["--".into()]).unwrap_or(extra);
    let argv = argv(context, name, extra)?;
    let spec = &context.config.commands[name];
    let runtime = context.path(&context.config.paths.runtime)?;
    let mut job = discipline_worker::jobs::Job::create(
        &runtime,
        &context.root,
        name,
        &context.path(&spec.cwd)?,
    )?;
    job.prepare(&argv, spec.read_only, spec.lifetime)?;
    let id = job.launch(&std::env::current_exe()?)?;
    println!("{}", serde_json::json!({"run_id": id}));
    Ok(0)
}
pub fn background_run(context: &Context, args: &[String]) -> Result<i32> {
    ensure!(args.len() == 1, "_job-run RUN_ID");
    let runtime = context.path(&context.config.paths.runtime)?;
    let mut job = discipline_worker::jobs::Job::adopt(&runtime, &args[0])?;
    let record = job.record();
    let (cwd, argv, read_only) = (record.cwd.clone(), record.argv.clone(), record.read_only);
    let result = process::tracked(&cwd, &argv, (read_only, false), Some(&mut job));
    let code = result.as_ref().map(process::exit_code).unwrap_or(127);
    job.finish(
        code,
        result.as_ref().err().map(|error| format!("{error:#}")),
    )?;
    result.map(|_| code)
}
