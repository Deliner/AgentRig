pub use super::settings::{
    AffectedTests, Agent, Api, Capabilities, Check, CheckKind, Command, Config, Containment, FILE,
    Frontend, Hooks, Oracle, Paths, Processes, Resource, Route, Runner, VERSION, Vcs, read,
    relative,
};
use anyhow::{Context as _, Result};
use std::path::{Path, PathBuf};

pub struct Context {
    pub root: PathBuf,
    pub config: Config,
}
impl Context {
    pub fn load(root: &Path) -> Result<Self> {
        Self::load_for(root, false)
    }
    pub fn load_for(root: &Path, recovery: bool) -> Result<Self> {
        let root = root.canonicalize()?;
        let mut config = read(&root)?;
        let pin = config.runtime.clone();
        let supported_pin = pin == VERSION || pin == super::settings::LEGACY_VERSION;
        let recovering = recovery && supported_pin && super::recovery::active(&root)?;
        if recovering {
            config.runtime = VERSION.into();
        }
        let validation = if recovering {
            config.validate_structure(&root)
        } else {
            config.validate(&root)
        };
        validation.with_context(|| {
            format!(
                "configuration invalid. ACTION: Apply {}",
                config.config_skill
            )
        })?;
        config.runtime = pin;
        Ok(Self { root, config })
    }
    pub fn path(&self, value: &str) -> Result<PathBuf> {
        relative(&self.root, value)
    }
}
