use super::Config;
use anyhow::Result;

pub const CODEX_CONFIG: &str = "[features]\nhooks = true\n";

fn binary(service: &str) -> String {
    format!(
        "\"$root\"/{}",
        shell_words::quote(&format!("{service}/bin/agentrig"))
    )
}

pub fn mcp_command(service: &str, command: &str) -> String {
    format!(
        "root=$(git rev-parse --show-toplevel) && exec {} {command} mcp --root \"$root\"",
        binary(service)
    )
}

pub fn git_hooks(config: &Config) -> [(String, Vec<u8>); 2] {
    let binary = binary(&config.paths.service);
    let prefix = "#!/bin/sh\nset -eu\nroot=$(git rev-parse --show-toplevel)\n";
    [
        (config.paths.service_path("hooks/pre-commit"), format!("{prefix}{binary} guard-commit --root \"$root\"\nexec {binary} check --root \"$root\" --staged\n").into_bytes()),
        (config.paths.service_path("hooks/reference-transaction"), format!("{prefix}exec {binary} guard-reference --root \"$root\" \"$1\"\n").into_bytes()),
    ]
}

pub fn registration(config: &Config) -> Result<Vec<u8>> {
    let command = format!(
        "root=$(git rev-parse --show-toplevel) && exec {} hook --root \"$root\"",
        binary(&config.paths.service)
    );
    let handler = serde_json::json!({"type": "command", "command": command, "timeout": 10});
    Ok(serde_json::to_vec_pretty(&serde_json::json!({"hooks": {
        "SessionStart": [{"matcher": "startup|resume|clear|compact", "hooks": [handler.clone()]}],
        "PreToolUse": [{"matcher": "Bash|Shell|exec_command|apply_patch|Edit|Write", "hooks": [handler]}]
    }}))?)
}
