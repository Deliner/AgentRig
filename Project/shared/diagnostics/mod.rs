// DECISION: D022
mod actionable_findings;
mod guidance;
mod rerun_command;
mod skill_refs;

use crate::shared::calling_agent::Agent;
use guidance::{configured, repair};
use rerun_command::{Invocation, executable, invocation};

pub(crate) use actionable_findings::print_diagnostics;
pub use actionable_findings::{Diagnostic, Level};
pub use guidance::{Guidance, UNNAMED_SKILL, caller, config_skill};
pub use rerun_command::rerun;
pub use skill_refs::validate_skill;

// DECISION: D188
pub fn failure(error: &anyhow::Error) -> String {
    let Invocation { root, args } = invocation();
    let command = args.first().map(String::as_str).unwrap_or("worker");
    let skill = configured(&root)
        .and_then(|config| repair(&root, &config, command, Agent::calling()))
        .unwrap_or_else(|| UNNAMED_SKILL.to_owned());
    let binary = executable();
    let rerun = shell_words::join(
        std::iter::once(binary.to_string_lossy().into_owned()).chain(args.iter().cloned()),
    );
    Guidance {
        level: "ERROR",
        id: command,
        location: &root.display().to_string(),
        message: &format!("{error:#}"),
        skill: &skill,
        rerun: &rerun,
    }
    .to_string()
}
