mod validation;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub profiles: BTreeMap<String, Profile>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub frontend: Frontend,
    pub model: String,
    pub reasoning_effort: String,
    pub mode: Mode,
    pub prompt: PathBuf,
    pub visible_paths: Vec<String>,
    pub timeout_seconds: u64,
    pub memory_bytes: Option<u64>,
    pub max_processes: Option<u64>,
    #[serde(default)]
    pub skills: Vec<PathBuf>,
    #[serde(default)]
    pub programs: BTreeMap<String, PathBuf>,
    #[serde(default)]
    pub mcp_servers: BTreeMap<String, McpServer>,
    pub credentials: Credentials,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Frontend {
    Codex,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    Read,
    Artifacts,
    Code,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct McpServer {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Credentials {
    pub codex_auth_file_env: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

pub fn load(path: &Path) -> Result<Config> {
    let source = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut config: Config = toml::from_str(&source).context("delegation configuration schema")?;
    ensure!(
        config.schema_version == 1,
        "unsupported delegation schema_version"
    );
    ensure!(
        !config.profiles.is_empty(),
        "delegation requires at least one profile"
    );
    for (name, profile) in &mut config.profiles {
        validation::profile(path, name, profile)
            .with_context(|| format!("delegate profile {name}"))?;
    }
    Ok(config)
}
