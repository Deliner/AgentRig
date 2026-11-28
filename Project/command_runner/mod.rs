// DECISION: D005
// DECISION: D107 allow-justification
#[expect(unsafe_code)]
mod jobs;
mod sandbox;
mod summary;
#[cfg(test)]
mod tests;
mod tool_registry;
pub use jobs::{
    Capture, Command as JobCommand, clean_jobs, cli as run_job_command,
    directory as jobs_directory, finished as finished_runs, list as list_jobs, owner as job_owner,
    streamed,
};
pub use sandbox::{CATALOG_ROOT, Sandboxing, available as sandbox_available, catalog_run, tracked};
pub use tool_registry::{PlacementSource, check_placement, list as list_tools};

use crate::project_config::{Access, Arguments, Command, Context, Program};
use crate::shared::console;
use anyhow::{Context as _, Result, bail, ensure};
use std::{collections::BTreeMap, path::PathBuf, process::Output};

pub fn argv(context: &Context, name: &str, extra: &[String]) -> Result<Vec<String>> {
    let command = context
        .config
        .commands
        .get(name)
        .with_context(|| format!("unknown command {name}"))?;
    accepted(name, command.accepts_args, extra)?;
    let argv: Vec<_> = command.argv.iter().chain(extra).cloned().collect();
    ensure!(!argv.is_empty(), "command {name} needs argv");
    permitted(name, &command.programs, &argv)?;
    Ok(argv)
}
// DECISION: D202
fn accepted(name: &str, arguments: Arguments, extra: &[String]) -> Result<()> {
    match arguments {
        Arguments::None => ensure!(extra.is_empty(), "command {name} does not accept arguments"),
        Arguments::Operands => {
            if let Some(option) = extra.iter().find(|word| word.starts_with('-')) {
                bail!(
                    "command {name} takes only names and paths; its program would read {option} as its own option"
                );
            }
        }
        Arguments::Any => {}
    }
    Ok(())
}
// DECISION: D203
fn access(command: &Command, argv: &[String]) -> Access {
    let program = argv.first().and_then(|name| command.programs.get(name));
    program
        .and_then(|program| program.access)
        .unwrap_or(command.access)
}
fn permitted(name: &str, programs: &BTreeMap<String, Program>, argv: &[String]) -> Result<()> {
    let unrestricted = programs.is_empty();
    if unrestricted {
        return Ok(());
    }
    let known = programs.keys().cloned().collect::<Vec<_>>().join(", ");
    let program = programs.get(&argv[0]).with_context(|| {
        format!(
            "{name} does not run {}; configured programs: {known}. Project commands: just list",
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
    let denied = argv[1..]
        .iter()
        .find(|arg| program.deny.iter().any(|flag| flag.denies(arg)));
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
        Capture::Collect
    } else {
        Capture::Stream
    };
    match logged(context, name, argv, capture) {
        Ok((output, log)) => {
            let code = jobs::exit_code(&output);
            if let Some(views) = views {
                summary::print(name, views, &output, log.as_deref());
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
    capture: Capture,
) -> Result<Output> {
    logged(context, name, argv, capture).map(|(output, _)| output)
}
pub fn output(context: &Context, argv: &[String]) -> Result<Output> {
    let options = Sandboxing {
        access: Access::Unconfined,
        capture: Capture::Collect,
        cargo_target: context.cargo_target.as_deref(),
        rustflags: &context.config.rust.rustflags,
    };
    tracked(&context.root, argv, options, None)
}
fn logged(
    context: &Context,
    name: &str,
    argv: Vec<String>,
    capture: Capture,
) -> Result<(Output, Option<PathBuf>)> {
    let spec = &context.config.commands[name];
    let cwd = context.path(&spec.cwd)?;
    let access = access(spec, &argv);
    let options = Sandboxing {
        access,
        capture,
        cargo_target: context.cargo_target.as_deref(),
        rustflags: &context.config.rust.rustflags,
    };
    let launch = jobs::Launch {
        command: name,
        cwd: &cwd,
        argv: &argv,
        access,
        lifetime: spec.lifetime,
    };
    let mut job = match jobs::Job::create(context, &launch) {
        Ok(job) => job,
        // DECISION: D193
        Err(refusal) => {
            let output = sandbox::confined(&cwd, &argv, options)?.ok_or(refusal)?;
            return Ok((output, None));
        }
    };
    let result = tracked(&cwd, &argv, options, Some(&mut job));
    let code = result.as_ref().map(jobs::exit_code).unwrap_or(127);
    job.finish(
        code,
        result.as_ref().err().map(|error| format!("{error:#}")),
    )?;
    result.map(|output| (output, Some(job.directory().to_owned())))
}
pub fn report(context: &Context) -> Result<()> {
    jobs::print_report(&context.runtime)
}
pub fn background(context: &Context, args: &[String]) -> Result<i32> {
    let (name, extra) = args
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("job-start COMMAND [-- ARGS]"))?;
    let extra = extra.trim_prefix(&["--".into()]);
    let argv = argv(context, name, extra)?;
    let spec = &context.config.commands[name];
    let cwd = context.path(&spec.cwd)?;
    let job = jobs::Job::create(
        context,
        &jobs::Launch {
            command: name,
            cwd: &cwd,
            argv: &argv,
            access: access(spec, &argv),
            lifetime: spec.lifetime,
        },
    )?;
    let id = job.launch(&std::env::current_exe()?)?;
    console::stdout(serde_json::json!({"run_id": id}));
    Ok(0)
}
pub fn background_run(context: &Context, args: &[String]) -> Result<i32> {
    ensure!(args.len() == 1, "_job-run RUN_ID");
    let runtime = context.runtime.clone();
    let mut job = jobs::Job::adopt(&runtime, &args[0])?;
    let record = job.record();
    let (cwd, argv, access) = (record.cwd.clone(), record.argv.clone(), record.access);
    let result = tracked(
        &cwd,
        &argv,
        Sandboxing {
            access,
            capture: Capture::Stream,
            cargo_target: None,
            rustflags: &context.config.rust.rustflags,
        },
        Some(&mut job),
    );
    let code = result.as_ref().map(jobs::exit_code).unwrap_or(127);
    job.finish(
        code,
        result.as_ref().err().map(|error| format!("{error:#}")),
    )?;
    result.map(|_| code)
}
