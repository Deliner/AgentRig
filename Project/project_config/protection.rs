// DECISION: D213
use super::settings_schema::{Config, FILE, module, relative};
use anyhow::{Context as _, Result};
use std::path::{Path, PathBuf};

const FIXED: [&str; 6] = [
    ".git",
    "justfile",
    ".agentrig/just",
    ".claude/settings.json",
    ".codex/hooks.json",
    ".codex/config.toml",
];
const MODULES: [&str; 3] = ["git", "commands", "hooks"];
const SERVICE: [&str; 2] = ["bin", "hooks"];
const RUNTIME: [&str; 2] = ["reminders", "approved"];

pub fn configuration() -> Vec<String> {
    std::iter::once(FILE.to_owned())
        .chain(MODULES.map(module))
        .collect()
}

pub struct Protection {
    paths: Vec<PathBuf>,
}
impl Protection {
    pub(super) fn of(root: &Path, config: &Config) -> Result<Self> {
        let settings = &config.paths;
        let mut named: Vec<String> = FIXED.map(String::from).into();
        named.extend(configuration());
        named.extend(SERVICE.map(|name| settings.service_path(name)));
        named.extend(RUNTIME.map(|name| format!("{}/{name}", settings.runtime)));
        let mut paths = Vec::new();
        for value in &named {
            paths.push(inside(root, value)?);
        }
        for value in &config.hooks.protected {
            paths.push(inside(root, value).context("hooks.protected")?);
        }
        if let Some(pin) = &config.git.pin {
            paths.push(inside(root, pin).context("git.pin")?);
        }
        Ok(Self { paths })
    }
    pub fn touched(&self, path: &Path) -> bool {
        self.paths
            .iter()
            .any(|protected| path.starts_with(protected) || protected.starts_with(path))
    }
}
fn inside(root: &Path, value: &str) -> Result<PathBuf> {
    Ok(relative(root, value)?.strip_prefix(root)?.to_owned())
}
