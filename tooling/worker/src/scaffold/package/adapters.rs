use super::Config;
use anyhow::Result;

pub const CODEX_CONFIG: &str = "[features]\nhooks = true\n";

fn binary(service: &str) -> String {
    format!(
        "\"$root\"/{}",
        shell_words::quote(&format!("{service}/bin/agentrig"))
    )
}

pub fn mcp_command(service: &str, command: &str, backend: review_runner::vcs::Kind) -> String {
    format!(
        "root=$({}) && exec {} {command} mcp --root \"$root\"",
        backend.root_command(),
        binary(service)
    )
}

pub fn environment_command(config: &Config, command: &str, name: &str) -> String {
    format!(
        "root=$({}) && exec {} environment-{command} {} --root \"$root\"",
        config.vcs.backend.root_command(),
        binary(&config.paths.service),
        shell_words::quote(name)
    )
}

pub fn vcs_hooks(config: &Config) -> Vec<(String, Vec<u8>)> {
    let binary = binary(&config.paths.service);
    config
        .vcs
        .backend
        .hooks(&binary)
        .into_iter()
        .map(|(name, contents)| {
            (
                config.paths.service_path(&format!("hooks/{name}")),
                contents.into_bytes(),
            )
        })
        .collect()
}

pub fn registration(config: &Config) -> Result<Vec<u8>> {
    let command = format!(
        "root=$({}) && exec {} hook --root \"$root\"",
        config.vcs.backend.root_command(),
        binary(&config.paths.service)
    );
    let handler = serde_json::json!({"type": "command", "command": command, "timeout": 10});
    let mut value = serde_json::json!({"hooks": {
        "SessionStart": [{"matcher": "startup|resume|clear|compact", "hooks": [handler.clone()]}],
        "PreToolUse": [{"matcher": "Bash|Shell|exec_command|apply_patch|Edit|Write", "hooks": [handler]}]
    }});
    let custom =
        agentrig::environment::hooks::configuration(&config.environment.hooks, |name, _| {
            environment_command(config, "hook", name)
        });
    for (event, groups) in custom.as_object().unwrap() {
        value["hooks"]
            .as_object_mut()
            .unwrap()
            .entry(event.clone())
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .unwrap()
            .extend(groups.as_array().unwrap().iter().cloned());
    }
    Ok(serde_json::to_vec_pretty(&value)?)
}
