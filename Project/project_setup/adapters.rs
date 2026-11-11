use super::Config;
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

pub fn registration(config: &Config, root_command: &str) -> Result<Vec<u8>> {
    let command = format!(
        "root=$({root_command}) && exec {binary} hook --root \"$root\"",
        binary = binary(&config.paths.service)
    );
    let handler = serde_json::json!({"type": "command", "command": command, "timeout": 10});
    let value = serde_json::json!({"hooks": {
        "SessionStart": [{"matcher": "startup|resume|clear|compact", "hooks": [handler.clone()]}],
        "PreToolUse": [{"matcher": "Bash|Shell|exec_command|apply_patch|Edit|Write", "hooks": [handler]}]
    }});
    Ok(serde_json::to_vec_pretty(&value)?)
}
