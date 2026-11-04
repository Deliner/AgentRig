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
    ENVIRONMENT.iter().any(|namespace| bounded(path, namespace))
}

fn bounded(path: &str, namespace: &str) -> bool {
    path == namespace
        || path
            .strip_prefix(namespace)
            .is_some_and(|rest| rest.starts_with("::"))
}

// DECISION: D083
pub const CLOCK: [&str; 2] = ["std::time::Instant::now", "std::time::SystemTime::now"];

// DECISION: D083
pub fn clock(external: &str) -> bool {
    CLOCK.contains(&external.trim_start_matches("::"))
}

// DECISION: D104
const EFFECTS: [(&str, &[&str]); 4] = [
    (
        "draw entropy",
        &[
            "rand::random",
            "rand::thread_rng",
            "rand::rng",
            "getrandom::getrandom",
            "getrandom::fill",
        ],
    ),
    ("use the network", &["std::net"]),
    ("use threads", &["std::thread"]),
    (
        "use the console",
        &[
            "std::io::stdin",
            "std::io::stdout",
            "std::io::stderr",
            "std::print",
            "std::println",
            "std::eprint",
            "std::eprintln",
        ],
    ),
];

// DECISION: D078
// DECISION: D083
// DECISION: D104
pub fn effect(external: &str) -> Option<&'static str> {
    let path = external.trim_start_matches("::");
    let environmental = environment(path);
    if environmental {
        return Some("access the environment");
    }
    let timekeeping = clock(path);
    if timekeeping {
        return Some("read the wall clock");
    }
    EFFECTS
        .iter()
        .find(|(_, namespaces)| namespaces.iter().any(|namespace| bounded(path, namespace)))
        .map(|(effect, _)| *effect)
}
