use crate::{
    change_delivery::{gate, git},
    command_runner,
    project_config::{self, Context},
    project_memory::{self, evidence},
    project_setup,
    shared::arguments::forwarded,
};

use anyhow::{Result, bail};
use std::path::Path;

pub fn owns(command: &str) -> bool {
    matches!(
        command,
        "config-check"
            | "run"
            | "commands"
            | "report"
            | "check"
            | "memory-check"
            | "resume"
            | "feature-start"
            | "feature-merge"
            | "init"
            | "doctor"
            | "jobs"
            | "job-status"
            | "job-logs"
            | "job-start"
            | "job-stop"
            | "job-cleanup"
            | "_job-run"
            | "scratch"
    )
}
pub fn run(root: &Path, command: &str, args: &[String]) -> Result<i32> {
    match command {
        "init" => return project_setup::initialization::init(root, args),
        "guard-commit" => return guard_commit(root, args),
        "check" => {
            return gate::selected(root, gate::arguments(args)?);
        }
        _ => {}
    }
    validate_arguments(command, args)?;
    let context = Context::load(root)?;
    let jobs = matches!(
        command,
        "jobs" | "job-status" | "job-logs" | "job-stop" | "job-cleanup"
    );
    if jobs {
        return crate::command_runner::jobs::cli(
            &context.path(&context.config.paths.runtime)?,
            command,
            args,
        );
    }
    match command {
        "job-start" => command_runner::background(&context, args),
        "_job-run" => command_runner::background_run(&context, args),
        "scratch" => command_runner::scratch::run(&context, args),
        _ => configured_command(&context, command, args),
    }
}
fn guard_commit(root: &Path, args: &[String]) -> Result<i32> {
    let unexpected = !args.is_empty();
    if unexpected {
        bail!("guard-commit takes no arguments");
    }
    let context = Context::load(root)?;
    git::guard_commit(root, &context.config.git)
}

fn validate_arguments(command: &str, args: &[String]) -> Result<()> {
    let unexpected = matches!(
        command,
        "doctor"
            | "memory-check"
            | "resume"
            | "report"
            | "config-check"
            | "commands"
            | "feature-merge"
    ) && !args.is_empty();
    if unexpected {
        bail!("{command} takes no arguments");
    }
    let invalid_start = command == "feature-start" && args.len() != 1;
    if invalid_start {
        bail!("feature-start NAME");
    }
    Ok(())
}
fn configured_command(context: &Context, command: &str, args: &[String]) -> Result<i32> {
    let root = &context.root;
    let git = &context.config.git;
    match command {
        "doctor" => project_setup::doctor(context),
        "config-check" => config_check(context),
        "guard-reference" => {
            git::guard_reference_with(root, args.first().map(String::as_str).unwrap_or(""), git)
        }
        "feature-start" => git::start(context, &args[0]),
        "feature-merge" => git::merge(context),
        "memory-check" => project_memory::check(context),
        "resume" => {
            project_memory::resume(context)?;
            Ok(0)
        }
        "commands" => {
            command_runner::registry::list(context)?;
            Ok(0)
        }
        "run" => {
            let (name, extra) = args
                .split_first()
                .ok_or_else(|| anyhow::anyhow!("run COMMAND [-- ARGS]"))?;
            run_command(context, name, forwarded(extra))
        }
        "report" => {
            command_runner::report(context)?;
            evidence::report(context)?;
            Ok(0)
        }
        _ => bail!("unknown scaffold command {command}"),
    }
}
fn run_command(context: &Context, name: &str, extra: &[String]) -> Result<i32> {
    let code = command_runner::run(context, name, extra)?;
    let failed = code != 0;
    let check = context
        .config
        .checks
        .iter()
        .find(|check| check.command.as_deref() == Some(name));
    if let Some(check) = check.filter(|_| failed) {
        let mut args = vec!["run".into(), name.into(), "--".into()];
        args.extend_from_slice(extra);
        let rerun = crate::shared::diagnostics::rerun(&context.root, &args);
        eprintln!(
            "{}",
            crate::shared::diagnostics::Guidance {
                level: "ERROR",
                id: &check.id,
                location: &context.config.commands[name].cwd,
                message: &format!("exited {code}; see original output above"),
                skill: &check.skill,
                rerun: &rerun
            }
        );
    }
    Ok(code)
}
fn config_check(context: &Context) -> Result<i32> {
    let code = if context.config.capabilities.lint {
        crate::structural_lint::run(
            &context.root,
            &context.path(&context.config.paths.lint)?,
            crate::structural_lint::Format::Json,
            crate::structural_lint::Mode::ConfigCheck,
        )?
    } else {
        0
    };
    let valid = code == 0;
    if valid {
        println!(
            "Scaffold configuration is valid (runtime {})",
            project_config::VERSION
        );
    }
    Ok(code)
}
