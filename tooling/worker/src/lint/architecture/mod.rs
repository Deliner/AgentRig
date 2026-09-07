//! Directory contracts and checks over resolved source dependencies.
mod graph;
mod inventory;
mod policy;
pub mod resolution;
pub mod runner;
mod settings;
pub mod source;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

pub use policy::Contracts;
pub use settings::Settings;

/// A source dependency resolved to normalized project-relative paths.
#[derive(Clone, Debug)]
pub struct Dependency {
    pub source: PathBuf,
    pub target: PathBuf,
    pub line: usize,
    /// Rust module ownership declarations still undergo boundary checks.
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
    /// Check measured edges; permission declarations are never graph edges.
    pub fn check(&self, dependencies: &[Dependency]) -> Vec<Issue> {
        let mut issues = Vec::new();
        for edge in dependencies {
            self.check_edge(edge, &mut issues);
        }
        issues.extend(graph::cycles(dependencies));
        issues
    }
}
