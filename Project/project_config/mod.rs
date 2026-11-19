pub(crate) mod settings_schema;
#[cfg(test)]
mod tests;
use crate::shared::calling_agent::{Agent, Caller};
use anyhow::Result;
pub use settings_schema::{
    Access, AffectedTests, Arguments, Capabilities, Check, CheckKind, Command, Config, FILE, Flag,
    Git, Hooks, Lifetime, Oracle, Paths, Program, Removed, Route, RouteEvent, Runner, Rust, Tools,
    VERSION, View, Views, files, module, read, relative, sources,
};
use std::path::{Path, PathBuf};

pub struct Context {
    pub root: PathBuf,
    pub config: Config,
    pub runtime: PathBuf,
    pub cargo_target: Option<PathBuf>,
    pub caller: Caller,
}
impl Context {
    pub fn load(root: &Path) -> Result<Self> {
        let root = settings_schema::project_root(root)?;
        let config = read(&root)?;
        config.validate(&root)?;
        let runtime = relative(&root, &config.paths.runtime)?;
        let caller = Caller {
            root: root.clone(),
            skills: config.paths.skills.clone().into(),
            agent: Agent::calling(),
        };
        Ok(Self {
            root,
            config,
            runtime,
            cargo_target: None,
            caller,
        })
    }
    pub fn path(&self, value: &str) -> Result<PathBuf> {
        relative(&self.root, value)
    }
}
