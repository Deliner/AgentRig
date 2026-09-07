use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, env};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Credentials {
    pub codex_auth_file_env: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

pub fn variable_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

pub fn validate_references(references: &BTreeMap<String, String>) -> Result<()> {
    for (name, reference) in references {
        ensure!(
            variable_name(name) && variable_name(reference),
            "credential environment entries must map variable names to variable names"
        );
        ensure!(
            ![
                "HOME",
                "CODEX_HOME",
                "CLAUDE_CONFIG_DIR",
                "PATH",
                "SHELL",
                "LD_PRELOAD",
                "LD_LIBRARY_PATH"
            ]
            .contains(&name.as_str()),
            "environment {name} is owned by the sandbox"
        );
    }
    Ok(())
}

pub fn resolve_references(
    references: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>> {
    references
        .iter()
        .map(|(name, reference)| {
            let value = env::var(reference)
                .with_context(|| format!("missing credential reference {reference}"))?;
            Ok((name.clone(), value))
        })
        .collect()
}

impl Credentials {
    pub fn validate_claude(&self, effort: &str) -> Result<()> {
        ensure!(
            self.codex_auth_file_env.is_none(),
            "claude-code does not use codex_auth_file_env; configure credentials.env"
        );
        ensure!(
            [
                "ANTHROPIC_API_KEY",
                "ANTHROPIC_AUTH_TOKEN",
                "CLAUDE_CODE_OAUTH_TOKEN"
            ]
            .iter()
            .any(|key| self.env.contains_key(*key)),
            "claude-code requires an ANTHROPIC_API_KEY, ANTHROPIC_AUTH_TOKEN or CLAUDE_CODE_OAUTH_TOKEN environment reference"
        );
        ensure!(
            ["low", "medium", "high", "xhigh", "max"].contains(&effort),
            "unsupported claude-code reasoning effort {effort}; use low, medium, high, xhigh or max"
        );
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(reference) = &self.codex_auth_file_env {
            ensure!(
                variable_name(reference),
                "codex_auth_file_env must name an environment variable"
            );
        }
        validate_references(&self.env)
    }
}
