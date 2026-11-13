// DECISION: D005
// DECISION: D107
#[expect(unsafe_code)]
pub mod jobs;
pub(crate) mod sandbox;
pub mod scratch;
mod summary;
#[cfg(test)]
mod tests;
pub mod tool_registry;
use crate::project_config::{Context, Program};
use crate::shared::console;
use anyhow::{Context as _, Result, bail, ensure};
use std::{collections::BTreeMap, path::PathBuf, process::Output};

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
    permitted(name, &command.programs, &argv)?;
    Ok(argv)
}
fn permitted(name: &str, programs: &BTreeMap<String, Program>, argv: &[String]) -> Result<()> {
    let unrestricted = programs.is_empty();
    if unrestricted {
        return Ok(());
    }
    let known = programs.keys().cloned().collect::<Vec<_>>().join(", ");
    let program = programs.get(&argv[0]).with_context(|| {
        format!(
            "{name} does not run {}; configured programs: {known}. Project commands: just list. Temporary code: just scratch -- TEXT",
            argv[0]
        )
    })?;
    let subcommand = argv.get(1).map_or("", String::as_str);
    let listed =
        program.subcommands.is_empty() || program.subcommands.iter().any(|item| item == subcommand);
    ensure!(
        listed,
        "{name} runs {} only with: {}",
        argv[0],
        program.subcommands.join(", ")
    );
    let denied = argv[1..].iter().find(|arg| {
        program
            .deny
            .iter()
            .any(|form| arg.starts_with(form.as_str()))
    });
    if let Some(arg) = denied {
        bail!("{name} does not run {} with {arg}", argv[0]);
    }
    Ok(())
}
pub fn run(context: &Context, name: &str, extra: &[String]) -> Result<i32> {
    let argv = argv(context, name, extra)?;
    let views = context.config.commands[name].output.as_ref();
    let summarized = views.is_some();
    let capture = if summarized {
        jobs::Capture::Collect
    } else {
        jobs::Capture::Stream
    };
    match logged(context, name, argv, capture) {
        Ok((output, log)) => {
            let code = jobs::process::exit_code(&output);
            if let Some(views) = views {
                summary::print(name, views, &output, &log);
            }
            Ok(code)
        }
        Err(error) => {
            console::stderr(format!("{error:#}"));
            Ok(127)
        }
    }
}
pub fn execute(
    context: &Context,
    name: &str,
    argv: Vec<String>,
    capture: jobs::Capture,
) -> Result<Output> {
    logged(context, name, argv, capture).map(|(output, _)| output)
}
fn logged(
    context: &Context,
    name: &str,
    argv: Vec<String>,
    capture: jobs::Capture,
) -> Result<(Output, PathBuf)> {
    let spec = &context.config.commands[name];
    let cwd = context.path(&spec.cwd)?;
    let mut job = crate::command_runner::jobs::Job::create(context, name, &cwd)?;
    job.prepare(&argv, spec.access, spec.lifetime)?;
    let result = sandbox::tracked(
        &cwd,
        &argv,
        sandbox::Sandboxing {
            access: spec.access,
            capture,
            cargo_target: context.cargo_target.as_deref(),
            rustflags: &context.config.rust.rustflags,
        },
        Some(&mut job),
    );
    let code = result.as_ref().map(jobs::process::exit_code).unwrap_or(127);
    job.finish(
        code,
        result.as_ref().err().map(|error| format!("{error:#}")),
    )?;
    result.map(|output| (output, job.directory().to_owned()))
}
pub fn report(context: &Context) -> Result<()> {
    crate::command_runner::jobs::report::print(&context.runtime)
}
pub fn background(context: &Context, args: &[String]) -> Result<i32> {
    let (name, extra) = args
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("job-start COMMAND [-- ARGS]"))?;
    let extra = extra.trim_prefix(&["--".into()]);
    let argv = argv(context, name, extra)?;
    let spec = &context.config.commands[name];
    let cwd = context.path(&spec.cwd)?;
    let mut job = crate::command_runner::jobs::Job::create(context, name, &cwd)?;
    job.prepare(&argv, spec.access, spec.lifetime)?;
    let id = job.launch(&std::env::current_exe()?)?;
    console::stdout(serde_json::json!({"run_id": id}));
    Ok(0)
}
pub fn background_run(context: &Context, args: &[String]) -> Result<i32> {
    ensure!(args.len() == 1, "_job-run RUN_ID");
    let runtime = context.runtime.clone();
    let mut job = crate::command_runner::jobs::Job::adopt(&runtime, &args[0])?;
    let record = job.record();
    let (cwd, argv, access) = (record.cwd.clone(), record.argv.clone(), record.access);
    let result = sandbox::tracked(
        &cwd,
        &argv,
        sandbox::Sandboxing {
            access,
            capture: jobs::Capture::Stream,
            cargo_target: None,
            rustflags: &context.config.rust.rustflags,
        },
        Some(&mut job),
    );
    let code = result.as_ref().map(jobs::process::exit_code).unwrap_or(127);
    job.finish(
        code,
        result.as_ref().err().map(|error| format!("{error:#}")),
    )?;
    result.map(|_| code)
}
