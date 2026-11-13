use crate::structural_lint::architecture::extraction::references::PropagatedCall;
use crate::structural_lint::architecture::{self as architecture, Disallowed, Issue};
use anyhow::Result;
use globset::GlobSet;
use std::path::Path;

// DECISION: D105
pub struct Policy<'a> {
    entries: Vec<(&'a Disallowed, GlobSet)>,
}

impl<'a> Policy<'a> {
    pub fn new(entries: &'a [Disallowed]) -> Result<Self> {
        let entries = entries
            .iter()
            .map(|entry| Ok((entry, entry.allowed()?)))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { entries })
    }

    pub fn check(&self, file: &Path, line: usize, external: &str) -> Option<Issue> {
        let path = external.trim_start_matches("::");
        let (entry, _) = self.entries.iter().find(|(entry, allowed)| {
            architecture::bounded(path, &[entry.path.as_str()]) && !allowed.is_match(file)
        })?;
        Some(Issue {
            path: file.into(),
            line: Some(line),
            message: format!(
                "`{path}` is disallowed here: {reason}; use {replacement}",
                reason = entry.reason,
                replacement = entry.replacement
            ),
        })
    }

    // DECISION: D125
    pub fn methods(&self, file: &Path, calls: &[PropagatedCall]) -> Vec<Issue> {
        calls
            .iter()
            .filter_map(|call| self.method(file, call))
            .collect()
    }

    fn method(&self, file: &Path, call: &PropagatedCall) -> Option<Issue> {
        let (entry, _) = self.entries.iter().find(|(entry, allowed)| {
            entry.methods.contains(&call.method) && !allowed.is_match(file)
        })?;
        Some(Issue {
            path: file.into(),
            line: Some(call.line),
            message: format!(
                "`.{method}()` with its error propagated is disallowed here: {reason}; use {replacement}",
                method = call.method,
                reason = entry.reason,
                replacement = entry.replacement
            ),
        })
    }
}
