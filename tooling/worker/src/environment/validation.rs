use super::{Environment, validate_references};
use anyhow::{Result, ensure};
use review_runner::config::{identifier, resource};
use std::{collections::BTreeSet, fs, os::unix::fs::PermissionsExt, path::Path};

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
