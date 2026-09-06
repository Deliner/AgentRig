use anyhow::Result;

pub const CODEX_CONFIG: &str = "[features]\nhooks = true\n";

pub fn git_hooks() -> [(&'static str, &'static [u8]); 2] {
    [
        (".worker/hooks/pre-commit", b"#!/bin/sh\nset -eu\nroot=$(git rev-parse --show-toplevel)\n\"$root/.worker/bin/agentrig\" guard-commit --root \"$root\"\nexec \"$root/.worker/bin/agentrig\" check --root \"$root\" --staged\n"),
        (".worker/hooks/reference-transaction", b"#!/bin/sh\nset -eu\nroot=$(git rev-parse --show-toplevel)\nexec \"$root/.worker/bin/agentrig\" guard-reference --root \"$root\" \"$1\"\n"),
    ]
}

pub fn registration() -> Result<Vec<u8>> {
    let command = "\"$(git rev-parse --show-toplevel)/.worker/bin/agentrig\" hook --root \"$(git rev-parse --show-toplevel)\"";
    let handler = serde_json::json!({"type": "command", "command": command, "timeout": 10});
    Ok(serde_json::to_vec_pretty(&serde_json::json!({"hooks": {
        "SessionStart": [{"matcher": "startup|resume|clear|compact", "hooks": [handler.clone()]}],
        "PreToolUse": [{"matcher": "Bash|Shell|exec_command|apply_patch|Edit|Write", "hooks": [handler]}]
    }}))?)
}
