// DECISION: D024
// DECISION: D023
use super::{Files, config::Config};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Ownership {
    Runtime,
    Asset,
    Configuration,
    Editable,
    Memory,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub sha256: String,
    pub ownership: Ownership,
    pub executable: bool,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub manifest_version: u32,
    pub package_version: String,
    pub config_schema: u32,
    pub files: BTreeMap<String, Entry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub local: BTreeMap<String, Option<String>>,
}
pub fn installed(files: &Files, config: &Config) -> Result<Vec<u8>> {
    let files = files
        .iter()
        .map(|(path, bytes)| {
            (
                path.clone(),
                Entry {
                    sha256: checksum(bytes),
                    ownership: ownership(path, config),
                    executable: executable(path),
                },
            )
        })
        .collect();
    let manifest = Manifest {
        manifest_version: 1,
        package_version: config.runtime.clone(),
        config_schema: config.version,
        files,
        local: BTreeMap::new(),
    };
    Ok(serde_json::to_vec_pretty(&manifest)?)
}
pub fn checksum(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn executable(path: &str) -> bool {
    std::path::Path::new(path)
        .components()
        .any(|component| component.as_os_str() == "bin" || component.as_os_str() == "hooks")
}
fn ownership(path: &str, config: &Config) -> Ownership {
    let memory = ["Plan", "State", "Decisions", "Invariants"]
        .iter()
        .any(|name| path == format!("{}/{name}.md", config.paths.memory));
    let settings = [
        "agentrig.yaml",
        ".codex/config.toml",
        ".claude/settings.json",
        ".mcp.json",
        &config.paths.lint,
    ]
    .contains(&path)
        || config.hooks.reminder.as_deref() == Some(path)
        || path.starts_with(&config.paths.service_path("review/config/"));
    let editable = (path.starts_with(&format!("{}/", config.paths.skills))
        && path.ends_with("/SKILL.md"))
        || config
            .environment
            .skills
            .iter()
            .any(|skill| std::path::Path::new(path).starts_with(skill))
        || path.starts_with(&config.paths.service_path("hooks/"))
        || path.starts_with(&config.paths.service_path("review/prompts/"))
        || ["AGENTS.md", "CLAUDE.md", "justfile", ".codex/hooks.json"].contains(&path);
    let runtime = path == config.paths.service_path("bin/agentrig");
    if memory {
        Ownership::Memory
    } else if settings {
        Ownership::Configuration
    } else if editable {
        Ownership::Editable
    } else if runtime {
        Ownership::Runtime
    } else {
        Ownership::Asset
    }
}

pub fn approved(context: &super::config::Context, path: &str, bytes: &[u8]) -> bool {
    let receipt = std::fs::read(
        context
            .root
            .join(context.config.paths.service_path("manifest.json")),
    )
    .ok()
    .and_then(|bytes| serde_json::from_slice::<Manifest>(&bytes).ok());
    receipt.is_some_and(|receipt| {
        receipt.package_version == super::config::VERSION
            && receipt.local.get(path) == Some(&Some(checksum(bytes)))
    })
}
