use super::Frontend;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api: Option<Api>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Api {
    pub key_env: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

impl Agent {
    pub(super) fn validate(&self, frontend: Frontend) -> Result<()> {
        if let Some(model) = &self.model {
            ensure!(!model.trim().is_empty(), "agent.model must not be empty");
        }
        if let Some(effort) = &self.reasoning_effort {
            let supported: &[&str] = match frontend {
                Frontend::Codex => &["minimal", "low", "medium", "high", "xhigh"],
                Frontend::ClaudeCode => &["low", "medium", "high", "xhigh"],
            };
            ensure!(
                supported.contains(&effort.as_str()),
                "agent.reasoning_effort: {} project settings support {}; session-only levels must be selected in the client",
                frontend.name(),
                supported.join(", ")
            );
        }
        if let Some(api) = &self.api {
            ensure!(
                review_runner::config::credentials::variable_name(&api.key_env),
                "agent.api.key_env must name an environment variable, not contain a key"
            );
            if let Some(url) = &api.base_url {
                let authority = url
                    .strip_prefix("https://")
                    .or_else(|| url.strip_prefix("http://"));
                ensure!(
                    authority.is_some_and(|rest| !rest.is_empty())
                        && !url.chars().any(char::is_whitespace),
                    "agent.api.base_url must be a nonempty HTTP(S) endpoint"
                );
            }
        }
        Ok(())
    }
}
