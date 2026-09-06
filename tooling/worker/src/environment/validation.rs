use super::Environment;
use anyhow::{Context, Result, ensure};
use review_runner::config::{identifier, resource};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    os::unix::fs::PermissionsExt,
    path::Path,
};

impl Environment {
    pub fn resolve(&mut self, config: &Path) -> Result<()> {
        self.resolve_skills(config)?;
        for (name, path) in &mut self.programs {
            ensure!(identifier(name), "invalid program name {name}");
            *path = resource(config, path)?;
            let metadata = fs::metadata(&path)?;
            ensure!(
                metadata.is_file() && metadata.permissions().mode() & 0o111 != 0,
                "program {name} must be an executable file"
            );
        }
        super::hooks::validate(&self.hooks, &self.programs)?;
        for (name, server) in &self.mcp_servers {
            ensure!(identifier(name), "invalid MCP server name {name}");
            ensure!(
                self.programs.contains_key(&server.program),
                "MCP server {name} references unknown program {}",
                server.program
            );
            validate_references(&server.env)?;
        }
        Ok(())
    }

    fn resolve_skills(&mut self, config: &Path) -> Result<()> {
        let mut names = BTreeSet::new();
        for skill in &mut self.skills {
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

pub fn variable_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
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
