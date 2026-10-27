pub(crate) mod settings;
use anyhow::Result;
pub use settings::{
    AffectedTests, Capabilities, Check, CheckKind, Command, Config, FILE, Frontend, Git, Hooks,
    Lifetime, Oracle, Paths, Program, Route, RouteEvent, Runner, VERSION, files, read, relative,
    sources,
};
use std::path::{Path, PathBuf};

pub struct Context {
    pub root: PathBuf,
    pub config: Config,
}
impl Context {
    pub fn load(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let config = read(&root)?;
        config.validate(&root)?;
        Ok(Self { root, config })
    }
    pub fn path(&self, value: &str) -> Result<PathBuf> {
        relative(&self.root, value)
    }
}
