// DECISION: D022
pub mod actionable_findings;
mod guidance;
mod rerun_command;
pub mod skill_refs;

use guidance::repair_skill;
use rerun_command::{Invocation, executable, invocation};

pub use guidance::{Guidance, config_skill};
pub use rerun_command::rerun;

pub fn failure(error: &anyhow::Error) -> String {
    let Invocation { root, args } = invocation();
    let command = args.first().map(String::as_str).unwrap_or("worker");
    let config =
        crate::shared::strict_yaml::read::<serde_json::Value>(&root.join(".agentrig/project.yaml"))
            .ok();
    let skill = repair_skill(config.as_ref(), command);
    let binary = executable();
    let rerun = shell_words::join(
        std::iter::once(binary.to_string_lossy().into_owned()).chain(args.iter().cloned()),
    );
    Guidance {
        level: "ERROR",
        id: command,
        location: &root.display().to_string(),
        message: &format!("{error:#}"),
        skill,
        rerun: &rerun,
    }
    .to_string()
}
