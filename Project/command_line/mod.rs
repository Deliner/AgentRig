use crate::{
    change_delivery::{gate, git},
    command_runner::{self, jobs},
    project_config::{self, Context},
    project_memory::{self, evidence},
    project_setup,
    shared::arguments::forwarded,
};

use anyhow::{Result, bail};
use std::path::Path;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Command {
    Init,
    GuardCommit,
    GuardReference,
    Check,
    Doctor,
    ConfigCheck,
    Commands,
    Run,
    Report,
    MemoryCheck,
    Resume,
    FeatureStart,
    FeatureMerge,
    Scratch,
    JobStart,
    JobRun,
    Jobs(jobs::Command),
}
const NAMES: &[(&str, Command)] = &[
    ("init", Command::Init),
    ("guard-commit", Command::GuardCommit),
    ("guard-reference", Command::GuardReference),
    ("check", Command::Check),
    ("doctor", Command::Doctor),
    ("config-check", Command::ConfigCheck),
    ("commands", Command::Commands),
    ("run", Command::Run),
    ("report", Command::Report),
    ("memory-check", Command::MemoryCheck),
    ("resume", Command::Resume),
    ("feature-start", Command::FeatureStart),
    ("feature-merge", Command::FeatureMerge),
    ("scratch", Command::Scratch),
    ("job-start", Command::JobStart),
    ("_job-run", Command::JobRun),
    ("jobs", Command::Jobs(jobs::Command::List)),
    ("job-status", Command::Jobs(jobs::Command::Status)),
    ("job-logs", Command::Jobs(jobs::Command::Logs)),
    ("job-stop", Command::Jobs(jobs::Command::Stop)),
    ("job-cleanup", Command::Jobs(jobs::Command::Cleanup)),
];
impl Command {
    pub fn parse(name: &str) -> Option<Self> {
        NAMES
            .iter()
            .find(|(known, _)| *known == name)
            .map(|(_, command)| *command)
    }
    fn name(self) -> &'static str {
        NAMES
            .iter()
            .find(|(_, command)| *command == self)
            .map_or("", |(name, _)| name)
    }
}
pub fn run(root: &Path, command: Command, args: &[String]) -> Result<i32> {
    match command {
        Command::Init => return project_setup::initialization::init(root, args),
        Command::GuardCommit => return guard_commit(root, args),
        Command::Check => {
            return gate::selected(root, gate::arguments(args)?);
        }
        _ => {}
    }
    validate_arguments(command, args)?;
    let context = Context::load(root)?;
    match command {
        Command::Jobs(job) => jobs::cli(&context.path(&context.config.paths.runtime)?, job, args),
        Command::JobStart => command_runner::background(&context, args),
        Command::JobRun => command_runner::background_run(&context, args),
        Command::Scratch => command_runner::scratch::run(&context, args),
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

fn validate_arguments(command: Command, args: &[String]) -> Result<()> {
    let unexpected = matches!(
        command,
        Command::Doctor
            | Command::MemoryCheck
            | Command::Resume
            | Command::Report
            | Command::ConfigCheck
            | Command::Commands
            | Command::FeatureMerge
    ) && !args.is_empty();
    if unexpected {
        bail!("{} takes no arguments", command.name());
    }
    let invalid_start = command == Command::FeatureStart && args.len() != 1;
    if invalid_start {
        bail!("feature-start NAME");
    }
    Ok(())
}
fn configured_command(context: &Context, command: Command, args: &[String]) -> Result<i32> {
    let root = &context.root;
    let git = &context.config.git;
    match command {
        Command::Doctor => project_setup::doctor(context),
        Command::ConfigCheck => config_check(context),
        Command::GuardReference => {
            git::guard_reference_with(root, args.first().map(String::as_str).unwrap_or(""), git)
        }
        Command::FeatureStart => git::start(context, &args[0]),
        Command::FeatureMerge => git::merge(context),
        Command::MemoryCheck => project_memory::check(context),
        Command::Resume => {
            project_memory::resume(context)?;
            Ok(0)
        }
        Command::Commands => {
            command_runner::registry::list(context)?;
            Ok(0)
        }
        Command::Run => {
            let (name, extra) = args
                .split_first()
                .ok_or_else(|| anyhow::anyhow!("run COMMAND [-- ARGS]"))?;
            run_command(context, name, forwarded(extra))
        }
        Command::Report => {
            command_runner::report(context)?;
            evidence::report(context)?;
            Ok(0)
        }
        _ => bail!("unknown scaffold command {}", command.name()),
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
