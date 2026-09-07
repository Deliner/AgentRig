use super::{Config, Files, config::Frontend};
use anyhow::Result;
use std::{collections::BTreeMap, path::Path};

pub(super) fn configured(
    root: &Path,
    config: &Config,
    files: &Files,
) -> Result<BTreeMap<&'static str, Frontend>> {
    let mut selected = BTreeMap::new();
    if let Some(review) = &config.capabilities.review {
        let source = super::setup::source(root, files, &review.config)?;
        let review: review_runner::config::Config = review_runner::config::yaml::decode(&source)?;
        for reviewer in review.reviewers.values() {
            match reviewer.frontend.as_str() {
                "claude-code" => selected.insert("REVIEW_CLAUDE_BIN", Frontend::ClaudeCode),
                _ => selected.insert("REVIEW_CODEX_BIN", Frontend::Codex),
            };
        }
    }
    if let Some(delegation) = &config.capabilities.delegation {
        let source = super::setup::source(root, files, &delegation.config)?;
        let delegation: agentrig::delegate::config::Config =
            review_runner::config::yaml::decode(&source)?;
        for profile in delegation.profiles.values() {
            match profile.frontend {
                Frontend::ClaudeCode => {
                    selected.insert("DELEGATE_CLAUDE_BIN", Frontend::ClaudeCode)
                }
                Frontend::Codex => selected.insert("DELEGATE_CODEX_BIN", Frontend::Codex),
            };
        }
    }
    Ok(selected)
}
