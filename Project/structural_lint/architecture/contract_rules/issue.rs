use super::dependency::Dependency;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Issue {
    pub path: PathBuf,
    pub line: Option<usize>,
    pub message: String,
}

impl Issue {
    pub(super) fn dependency(edge: &Dependency, message: String) -> Self {
        Self {
            path: edge.source.clone(),
            line: Some(edge.line),
            message,
        }
    }
}
