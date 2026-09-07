use super::{Credentials, Frontend, Profile};
use anyhow::{Result, ensure};
use review_runner::config::{globs, identifier, resource};
use std::{fs, path::Path};

pub(super) fn profile(config: &Path, name: &str, profile: &mut Profile) -> Result<()> {
    ensure!(identifier(name), "invalid profile identifier");
    ensure!(!profile.model.trim().is_empty(), "model must not be empty");
    match profile.frontend {
        Frontend::Codex => review_runner::config::reasoning_effort(&profile.reasoning_effort)?,
        Frontend::ClaudeCode => profile
            .credentials
            .validate_claude(&profile.reasoning_effort)?,
    }
    ensure!(
        profile.timeout_seconds > 0,
        "timeout_seconds must be positive"
    );
    ensure!(
        profile.memory_bytes != Some(0),
        "memory_bytes must be positive"
    );
    ensure!(
        profile.max_processes != Some(0),
        "max_processes must be positive"
    );
    globs(&profile.visible_paths)?;
    profile.prompt = resource(config, &profile.prompt)?;
    ensure!(
        !fs::read_to_string(&profile.prompt)?.trim().is_empty(),
        "prompt must not be empty"
    );
    profile.environment.resolve(config)?;
    match profile.frontend {
        Frontend::Codex => credentials(&profile.credentials),
        Frontend::ClaudeCode => profile.credentials.validate(),
    }
}

fn credentials(credentials: &Credentials) -> Result<()> {
    let authenticated =
        credentials.codex_auth_file_env.is_some() || credentials.env.contains_key("OPENAI_API_KEY");
    ensure!(
        authenticated,
        "credentials requires codex_auth_file_env or an OPENAI_API_KEY environment reference"
    );
    if let Some(name) = &credentials.codex_auth_file_env {
        ensure!(
            crate::environment::variable_name(name),
            "codex_auth_file_env must name an environment variable"
        );
    }
    crate::environment::validate_references(&credentials.env)
}
