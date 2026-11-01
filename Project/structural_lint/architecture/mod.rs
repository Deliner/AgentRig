mod features;
mod graph;
mod inventory;
mod policy;
pub mod resolution;
pub mod runner;
pub mod source;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

pub use super::rules::architecture::{PYTHON_SEED, RUST_SEED, Settings};
pub use policy::{Contracts, Loaded, describe};

#[derive(Clone, Debug)]
pub struct Dependency {
    pub source: PathBuf,
    pub target: PathBuf,
    pub line: usize,
    pub module_declaration: bool,
}

#[derive(Debug)]
pub struct Issue {
    pub path: PathBuf,
    pub line: Option<usize>,
    pub message: String,
}

impl Issue {
    fn dependency(edge: &Dependency, message: String) -> Self {
        Self {
            path: edge.source.clone(),
            line: Some(edge.line),
            message,
        }
    }
}

// DECISION: D078
pub const ENVIRONMENT: [&str; 3] = ["std::env", "std::fs", "std::process"];

// DECISION: D078
#[derive(Debug)]
pub struct EnvironmentUse {
    pub line: usize,
    pub path: String,
}

// DECISION: D078
pub fn environment(external: &str) -> bool {
    let path = external.trim_start_matches("::");
    ENVIRONMENT.iter().any(|namespace| {
        path == *namespace
            || path
                .strip_prefix(namespace)
                .is_some_and(|rest| rest.starts_with("::"))
    })
}

// DECISION: D083
pub const CLOCK: [&str; 2] = ["std::time::Instant::now", "std::time::SystemTime::now"];

// DECISION: D083
pub fn clock(external: &str) -> bool {
    CLOCK.contains(&external.trim_start_matches("::"))
}
