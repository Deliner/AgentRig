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
    job.prepare(&argv)?;
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
