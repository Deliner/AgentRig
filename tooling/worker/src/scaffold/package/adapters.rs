use super::Config;
use anyhow::Result;
use std::path::Path;

pub const CODEX_CONFIG: &str = "[features]\nhooks = true\n";

fn binary(service: &str) -> String {
    format!(
        "\"$root\"/{}",
        shell_words::quote(&format!("{service}/bin/agentrig"))
    )
}

pub fn mcp_command(service: &str, command: &str, root_command: &str) -> String {
    format!(
        "root=$({}) && exec {} {command} mcp --root \"$root\"",
        root_command,
        binary(service)
    )
}

pub fn environment_command(service: &str, command: &str, name: &str, root_command: &str) -> String {
    format!(
        "root=$({}) && exec {} environment-{command} {} --root \"$root\"",
        root_command,
        binary(service),
        shell_words::quote(name)
    )
}

pub fn generated(root: &Path, config: &Config) -> Result<review_runner::vcs::Generated> {
    let ignored = vec![
        config.paths.runtime.clone(),
        config.paths.service_path("review/runtime"),
        config.paths.service_path("review/reports"),
    ];
    config.vcs.backend.generate(
        root,
        &review_runner::vcs::Generation {
            binary: &binary(&config.paths.service),
            directory: &config.paths.service_path("hooks"),
            ignored: &ignored,
        },
    )
}

pub fn vcs_hooks(root: &Path, config: &Config) -> Result<Vec<(String, Vec<u8>)>> {
    let prefix = config.paths.service_path("hooks/");
    Ok(generated(root, config)?
        .files
        .into_iter()
        .filter(|(path, _)| path.starts_with(&prefix))
        .map(|(path, contents)| (path, contents.into_bytes()))
        .collect())
}

pub fn registration(config: &Config, root_command: &str) -> Result<Vec<u8>> {
    let command = format!(
        "root=$({}) && exec {} hook --root \"$root\"",
        root_command,
        binary(&config.paths.service)
    );
    let handler = serde_json::json!({"type": "command", "command": command, "timeout": 10});
    let mut value = serde_json::json!({"hooks": {
        "SessionStart": [{"matcher": "startup|resume|clear|compact", "hooks": [handler.clone()]}],
        "PreToolUse": [{"matcher": "Bash|Shell|exec_command|apply_patch|Edit|Write", "hooks": [handler]}]
    }});
    let custom = crate::environment::hooks::configuration(&config.environment.hooks, |name, _| {
        environment_command(&config.paths.service, "hook", name, root_command)
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
