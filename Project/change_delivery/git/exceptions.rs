// DECISION: D220
use crate::shared::vcs::{Entry, FileKind, Repository};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Key {
    path: String,
    decision: String,
    rule: String,
    text: String,
}
pub(super) struct Added {
    pub(super) key: Key,
    line: usize,
}
impl Added {
    pub(super) fn describe(&self) -> String {
        let Key {
            path,
            decision,
            rule,
            text,
        } = &self.key;
        let line = self.line;
        format!("{path}:{line} {decision} {rule} above {text}")
    }
}

pub(super) struct Contents<'a> {
    pub(super) current: &'a BTreeMap<String, Entry>,
    pub(super) committed: &'a BTreeMap<String, Entry>,
}

pub(super) fn added(
    repository: &Repository<'_>,
    paths: &[String],
    contents: Contents<'_>,
) -> Result<Vec<Added>> {
    let mut added = Vec::new();
    for path in paths {
        let before = keyed(repository, path, contents.committed.get(path))?;
        let after = keyed(repository, path, contents.current.get(path))?;
        added.extend(beyond(after, before.iter().map(|found| &found.key)));
    }
    Ok(added)
}
pub(super) fn unapproved(added: Vec<Added>, approved: &[Key]) -> Vec<Added> {
    beyond(added, approved)
}
fn beyond<'a>(found: Vec<Added>, known: impl IntoIterator<Item = &'a Key>) -> Vec<Added> {
    let mut counts: BTreeMap<&Key, usize> = BTreeMap::new();
    for key in known {
        *counts.entry(key).or_default() += 1;
    }
    found
        .into_iter()
        .filter(|item| match counts.get_mut(&item.key) {
            Some(count) if *count > 0 => {
                *count -= 1;
                false
            }
            _ => true,
        })
        .collect()
}
fn keyed(repository: &Repository<'_>, path: &str, entry: Option<&Entry>) -> Result<Vec<Added>> {
    let Some(entry) = entry else {
        return Ok(Vec::new());
    };
    match entry.kind {
        FileKind::File | FileKind::Executable => {}
        FileKind::Symlink | FileKind::Submodule => return Ok(Vec::new()),
    }
    let content = repository.run(&["cat-file", "blob", &entry.object])?;
    let Ok(source) = String::from_utf8(content) else {
        return Ok(Vec::new());
    };
    let lines: Vec<&str> = source.lines().collect();
    let exceptions = crate::shared::source_parsers::exceptions(Path::new(path), &source)?;
    Ok(exceptions
        .into_iter()
        .map(|exception| Added {
            line: exception.line,
            key: Key {
                path: path.to_owned(),
                decision: exception.decision,
                rule: exception.rule,
                text: lines
                    .get(exception.line)
                    .map_or("", |line| line.trim())
                    .to_owned(),
            },
        })
        .collect())
}
