pub(crate) mod settings;
#[cfg(test)]
mod tests;
use anyhow::Result;
pub use settings::{
    Access, AffectedTests, Capabilities, Check, CheckKind, Command, Config, FILE, Frontend, Git,
    Hooks, Lifetime, Oracle, Paths, Program, Removed, Route, RouteEvent, Runner, Tools, VERSION,
    View, Views, access, files, module, read, relative, sources,
};
use std::path::{Path, PathBuf};

pub struct Context {
    pub root: PathBuf,
    pub config: Config,
    pub runtime: PathBuf,
}
impl Context {
    pub fn load(root: &Path) -> Result<Self> {
        let root = settings::project_root(root)?;
        let config = read(&root)?;
        config.validate(&root)?;
        let runtime = relative(&root, &config.paths.runtime)?;
        Ok(Self {
            root,
            config,
            runtime,
        })
    }
    pub fn path(&self, value: &str) -> Result<PathBuf> {
        relative(&self.root, value)
    }
}
