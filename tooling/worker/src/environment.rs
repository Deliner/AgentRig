pub mod hooks;
mod validation;
pub use validation::{resolve_references, validate_references, variable_name};

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    #[serde(default)]
    pub skills: Vec<PathBuf>,
    #[serde(default)]
    pub programs: BTreeMap<String, PathBuf>,
    #[serde(default)]
    pub mcp_servers: BTreeMap<String, McpServer>,
    #[serde(default)]
    pub hooks: BTreeMap<String, hooks::Hook>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct McpServer {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

impl Environment {
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
            && self.programs.is_empty()
            && self.mcp_servers.is_empty()
            && self.hooks.is_empty()
    }
}

// Keep nested field diagnostics when an existing profile flattens this declaration.
pub fn deserialize<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Environment, D::Error> {
    serde_path_to_error::deserialize(deserializer).map_err(serde::de::Error::custom)
}
