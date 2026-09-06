use super::{Credentials, Profile};
use anyhow::{Result, ensure};
use review_runner::config::{globs, identifier, resource};
use std::{collections::BTreeSet, fs, os::unix::fs::PermissionsExt, path::Path};

pub(super) fn profile(config: &Path, name: &str, profile: &mut Profile) -> Result<()> {
    ensure!(identifier(name), "invalid profile identifier");
    ensure!(!profile.model.trim().is_empty(), "model must not be empty");
    review_runner::config::reasoning_effort(&profile.reasoning_effort)?;
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
    skills(config, &mut profile.skills)?;
    programs(config, profile)?;
    credentials(&profile.credentials)?;
    for (name, server) in &profile.mcp_servers {
        ensure!(identifier(name), "invalid MCP server name {name}");
        ensure!(
            profile.programs.contains_key(&server.program),
            "MCP server {name} references unknown program {}",
            server.program
        );
        environment(&server.env)?;
    }
    Ok(())
}

fn skills(config: &Path, skills: &mut [std::path::PathBuf]) -> Result<()> {
    let mut names = BTreeSet::new();
    for skill in skills {
        *skill = resource(config, skill)?;
        let name = skill
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        ensure!(
            identifier(name) && names.insert(name.to_owned()),
            "invalid or duplicate skill directory name {name}"
        );
        fs::read_to_string(skill.join("SKILL.md"))?;
    }
    Ok(())
}

fn programs(config: &Path, profile: &mut Profile) -> Result<()> {
    for (name, path) in &mut profile.programs {
        ensure!(identifier(name), "invalid program name {name}");
        *path = resource(config, path)?;
        let metadata = fs::metadata(&path)?;
        ensure!(
            metadata.is_file() && metadata.permissions().mode() & 0o111 != 0,
            "program {name} must be an executable file"
        );
    }
    Ok(())
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
            env_name(name),
            "codex_auth_file_env must name an environment variable"
        );
    }
    environment(&credentials.env)
}

fn environment(env: &std::collections::BTreeMap<String, String>) -> Result<()> {
    for (name, reference) in env {
        ensure!(
            env_name(name) && env_name(reference),
            "credential environment entries must map variable names to variable names"
        );
        ensure!(
            ![
                "HOME",
                "CODEX_HOME",
                "PATH",
                "LD_PRELOAD",
                "LD_LIBRARY_PATH"
            ]
            .contains(&name.as_str()),
            "environment {name} is owned by the sandbox"
        );
    }
    Ok(())
}

fn env_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
