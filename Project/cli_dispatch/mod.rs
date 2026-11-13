mod project;
#[cfg(test)]
mod tests;
use crate::{
    change_delivery::{gate, git},
    command_runner::{self, jobs},
    project_config::Context,
    project_setup,
};

use anyhow::{Result, bail};
use std::path::Path;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Command {
    Init,
    GuardCommit,
    Check,
    Runner(Runner),
    Configured(Configured),
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Runner {
    Scratch,
    JobStart,
    JobRun,
    Jobs(jobs::Command),
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Configured {
    RuntimeAdd,
    GuardReference,
    Doctor,
    ConfigCheck,
    Commands,
    Run,
    Report,
    MemoryCheck,
    Resume,
    FeatureStart,
    FeatureMerge,
}
const NAMES: &[(&str, Command)] = &[
    ("init", Command::Init),
    ("runtime-add", Command::Configured(Configured::RuntimeAdd)),
    ("guard-commit", Command::GuardCommit),
    (
        "guard-reference",
        Command::Configured(Configured::GuardReference),
    ),
    ("check", Command::Check),
    ("doctor", Command::Configured(Configured::Doctor)),
    ("config-check", Command::Configured(Configured::ConfigCheck)),
    ("commands", Command::Configured(Configured::Commands)),
    ("run", Command::Configured(Configured::Run)),
    ("report", Command::Configured(Configured::Report)),
    ("memory-check", Command::Configured(Configured::MemoryCheck)),
    ("resume", Command::Configured(Configured::Resume)),
    (
        "feature-start",
        Command::Configured(Configured::FeatureStart),
    ),
    (
        "feature-merge",
        Command::Configured(Configured::FeatureMerge),
    ),
    ("scratch", Command::Runner(Runner::Scratch)),
    ("job-start", Command::Runner(Runner::JobStart)),
    ("_job-run", Command::Runner(Runner::JobRun)),
    ("jobs", Command::Runner(Runner::Jobs(jobs::Command::List))),
    (
        "job-status",
        Command::Runner(Runner::Jobs(jobs::Command::Status)),
    ),
    (
        "job-logs",
        Command::Runner(Runner::Jobs(jobs::Command::Logs)),
    ),
    (
        "job-stop",
        Command::Runner(Runner::Jobs(jobs::Command::Stop)),
    ),
    (
        "job-cleanup",
        Command::Runner(Runner::Jobs(jobs::Command::Cleanup)),
    ),
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
        Command::Init => project_setup::initialization::init(root, args),
        Command::GuardCommit => guard_commit(root, args),
        Command::Check => gate::selected(root, gate::arguments(args)?),
        Command::Runner(runner) => run_runner(&Context::load(root)?, runner, args),
        Command::Configured(configured) => {
            validate_arguments(configured, args)?;
            project::configured_command(&Context::load(root)?, configured, args)
        }
    }
}
fn run_runner(context: &Context, command: Runner, args: &[String]) -> Result<i32> {
    match command {
        Runner::Scratch => command_runner::scratch::run(context, args),
        Runner::JobStart => command_runner::background(context, args),
        Runner::JobRun => command_runner::background_run(context, args),
        Runner::Jobs(job) => jobs::cli(&context.path(&context.config.paths.runtime)?, job, args),
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

fn validate_arguments(command: Configured, args: &[String]) -> Result<()> {
    let usage = match command {
        Configured::Doctor
        | Configured::RuntimeAdd
        | Configured::MemoryCheck
        | Configured::Resume
        | Configured::Report
        | Configured::ConfigCheck
        | Configured::Commands
        | Configured::FeatureMerge => (!args.is_empty()).then(|| {
            format!(
                "{name} takes no arguments",
                name = Command::Configured(command).name()
            )
        }),
        Configured::FeatureStart => (args.len() != 1).then(|| "feature-start NAME".to_owned()),
        Configured::GuardReference | Configured::Run => None,
    };
    if let Some(usage) = usage {
        bail!(usage);
    }
    Ok(())
}
