use crate::command_runner::JobCommand;
use anyhow::{Result, bail};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Command {
    Init,
    GuardCommit,
    GuardReference,
    Check,
    Runner(Runner),
    Configured(Configured),
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Runner {
    JobStart,
    JobRun,
    Jobs(JobCommand),
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Configured {
    RuntimeAdd,
    Doctor,
    ConfigCheck,
    Commands,
    Run,
    Report,
    MemoryCheck,
    Resume,
    FeatureStart,
    FeatureMerge,
    FeaturePush,
    StateReset,
    Approve,
    ProposePin,
    Consent,
    RevokeConsent,
    ConsentedPins,
}
const NAMES: &[(&str, Command)] = &[
    ("init", Command::Init),
    ("runtime-add", Command::Configured(Configured::RuntimeAdd)),
    ("guard-commit", Command::GuardCommit),
    ("guard-reference", Command::GuardReference),
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
    ("feature-push", Command::Configured(Configured::FeaturePush)),
    ("state-reset", Command::Configured(Configured::StateReset)),
    // DECISION: D213
    ("approve", Command::Configured(Configured::Approve)),
    // DECISION: D218
    ("propose-pin", Command::Configured(Configured::ProposePin)),
    // DECISION: D226
    ("consent", Command::Configured(Configured::Consent)),
    (
        "revoke-consent",
        Command::Configured(Configured::RevokeConsent),
    ),
    (
        "consented-pins",
        Command::Configured(Configured::ConsentedPins),
    ),
    ("job-start", Command::Runner(Runner::JobStart)),
    ("_job-run", Command::Runner(Runner::JobRun)),
    ("jobs", Command::Runner(Runner::Jobs(JobCommand::List))),
    (
        "job-status",
        Command::Runner(Runner::Jobs(JobCommand::Status)),
    ),
    ("job-logs", Command::Runner(Runner::Jobs(JobCommand::Logs))),
    ("job-stop", Command::Runner(Runner::Jobs(JobCommand::Stop))),
    (
        "job-cleanup",
        Command::Runner(Runner::Jobs(JobCommand::Cleanup)),
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

pub(super) fn validate_arguments(command: Configured, args: &[String]) -> Result<()> {
    let usage = match command {
        Configured::Doctor
        | Configured::RuntimeAdd
        | Configured::MemoryCheck
        | Configured::Resume
        | Configured::Report
        | Configured::ConfigCheck
        | Configured::Commands
        | Configured::FeatureMerge
        | Configured::FeaturePush
        | Configured::StateReset
        | Configured::Approve
        | Configured::Consent
        | Configured::RevokeConsent
        | Configured::ConsentedPins => (!args.is_empty()).then(|| {
            format!(
                "{name} takes no arguments",
                name = Command::Configured(command).name()
            )
        }),
        Configured::FeatureStart => (args.len() != 1).then(|| "feature-start NAME".to_owned()),
        Configured::ProposePin => (args.len() != 1).then(|| "propose-pin REVISION".to_owned()),
        Configured::Run => None,
    };
    if let Some(usage) = usage {
        bail!(usage);
    }
    Ok(())
}
