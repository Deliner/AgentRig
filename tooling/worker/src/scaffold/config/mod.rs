pub use super::settings::{
    AffectedTests, Capabilities, Check, CheckKind, Command, Config, FILE, Frontend, Git, Hooks,
    Oracle, Paths, Route, Runner, VERSION, read, relative,
};
use anyhow::{Context as _, Result};
use std::path::{Path, PathBuf};

pub struct Context {
    pub root: PathBuf,
    pub config: Config,
}
impl Context {
    pub fn load(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let config = read(&root)?;
        config.validate(&root).with_context(|| {
            format!(
                "configuration invalid. ACTION: Apply {}",
                config.config_skill
            )
        })?;
        Ok(Self { root, config })
    }
    pub fn path(&self, value: &str) -> Result<PathBuf> {
        relative(&self.root, value)
    }
}
