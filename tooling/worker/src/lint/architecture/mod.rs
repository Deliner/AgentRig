mod graph;
mod inventory;
mod policy;
pub mod resolution;
pub mod runner;
pub mod source;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

pub use super::rules::architecture::Settings;
pub use policy::Contracts;

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

impl Contracts {
    pub fn check(&self, dependencies: &[Dependency]) -> Vec<Issue> {
        let mut issues = Vec::new();
        for edge in dependencies {
            self.check_edge(edge, &mut issues);
        }
        issues.extend(graph::cycles(dependencies));
        issues
    }
}
