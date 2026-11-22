use crate::project_config::Config;
use crate::shared::calling_agent::Agent;
use anyhow::Result;
use std::collections::BTreeMap;

pub const CODEX_CONFIG: &str = "[features]\nhooks = true\n";
const ROOT_COMMAND: &str = "git rev-parse --show-toplevel";

fn launcher(service: &str) -> String {
    shell_words::quote(&format!("{service}/bin/agentrig")).into_owned()
}

fn binary(service: &str) -> String {
    format!("\"$root\"/{}", launcher(service))
}

pub fn generated(config: &Config) -> Result<BTreeMap<String, String>> {
    crate::shared::vcs::generate(&crate::shared::vcs::Generation {
        binary: &launcher(&config.paths.service),
        directory: &config.paths.service_path("hooks"),
    })
}

pub fn vcs_hooks(config: &Config) -> Result<Vec<(String, Vec<u8>)>> {
    let prefix = config.paths.service_path("hooks/");
    Ok(generated(config)?
        .into_iter()
        .filter(|(path, _)| path.starts_with(&prefix))
        .map(|(path, contents)| (path, contents.into_bytes()))
        .collect())
}

// DECISION: D188
// DECISION: D200
pub fn settings(config: &Config, agent: Agent) -> serde_json::Value {
    let command = format!(
        "root=$({ROOT_COMMAND}) && {binary} hook --root \"$root\" --agent {agent} || {{ echo 'AgentRig hook could not run; repair its launcher and runtime' >&2; exit 2; }}",
        binary = binary(&config.paths.service),
        agent = agent.name()
    );
    let handler = serde_json::json!({"type": "command", "command": command, "timeout": 30,
        "statusMessage": "Loading worker context and edit guidance"});
    serde_json::json!({"hooks": {
        "SessionStart": [{"matcher": "startup|resume|clear|compact", "hooks": [handler.clone()]}],
        "PreToolUse": [{"matcher": "*", "hooks": [handler]}]
    }})
}

pub fn registration(config: &Config, agent: Agent) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec_pretty(&settings(config, agent))?)
}

pub fn hooks_registered(
    config: &Config,
    configured: Option<&serde_json::Value>,
    agent: Agent,
) -> bool {
    let expected = settings(config, agent);
    configured.is_some_and(|value| {
        expected["hooks"].as_object().is_some_and(|hooks| {
            hooks.iter().all(|(event, desired)| {
                value["hooks"][event].as_array().is_some_and(|routes| {
                    desired
                        .as_array()
                        .is_some_and(|items| items.iter().all(|route| routes.contains(route)))
                })
            })
        })
    })
}
