mod project;
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
        _ => project::configured_command(&context, command, args),
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
