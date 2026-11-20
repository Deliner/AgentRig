// DECISION: D226
use crate::shared::vcs::Repository;
use anyhow::{Context as _, Result};
use serde_json::{Map, Value};
use std::path::Path;

const RULES: &str = "rules";

pub fn only_adds_rules(repository: &Repository<'_>, relative: &Path) -> Result<bool> {
    let mut committed = settings(repository, "HEAD", relative)?;
    let mut staged = settings(repository, "", relative)?;
    let kept = committed.remove(RULES);
    let extended = staged.remove(RULES);
    let (Some(Value::Array(kept)), Some(Value::Array(extended))) = (kept, extended) else {
        return Ok(false);
    };
    let mut remaining = extended.iter();
    let preserved = kept
        .iter()
        .all(|rule| remaining.any(|candidate| candidate == rule));
    Ok(preserved && committed == staged)
}

fn settings(
    repository: &Repository<'_>,
    revision: &str,
    relative: &Path,
) -> Result<Map<String, Value>> {
    let source = String::from_utf8(repository.read(revision, relative)?)?;
    crate::shared::strict_yaml::decode(&source)
        .with_context(|| format!("lint configuration {revision}:{}", relative.display()))
}
