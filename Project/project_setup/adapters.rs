use super::Config;
use crate::shared::calling_agent::Agent;
use anyhow::Result;

pub const CODEX_CONFIG: &str = "[features]\nhooks = true\n";

fn binary(service: &str) -> String {
    format!(
        "\"$root\"/{}",
        shell_words::quote(&format!("{service}/bin/agentrig"))
    )
}

pub fn generated(config: &Config) -> Result<crate::shared::vcs::Generated> {
    crate::shared::vcs::generate(&crate::shared::vcs::Generation {
        binary: &binary(&config.paths.service),
        directory: &config.paths.service_path("hooks"),
    })
}

pub fn vcs_hooks(config: &Config) -> Result<Vec<(String, Vec<u8>)>> {
    let prefix = config.paths.service_path("hooks/");
    Ok(generated(config)?
        .files
        .into_iter()
        .filter(|(path, _)| path.starts_with(&prefix))
        .map(|(path, contents)| (path, contents.into_bytes()))
        .collect())
}

// DECISION: D188
// DECISION: D200
pub fn registration(config: &Config, root_command: &str, agent: Agent) -> Result<Vec<u8>> {
    let command = format!(
        "root=$({root_command}) && {binary} hook --root \"$root\" --agent {agent} || {{ echo 'AgentRig hook could not run; repair its launcher and runtime' >&2; exit 2; }}",
        binary = binary(&config.paths.service),
        agent = agent.name()
    );
    let handler = serde_json::json!({"type": "command", "command": command, "timeout": 30,
        "statusMessage": "Loading worker context and edit guidance"});
    let value = serde_json::json!({"hooks": {
        "SessionStart": [{"matcher": "startup|resume|clear|compact", "hooks": [handler.clone()]}],
        "PreToolUse": [{"matcher": "*", "hooks": [handler]}]
    }});
    Ok(serde_json::to_vec_pretty(&value)?)
}
