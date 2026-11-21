// DECISION: D220
use crate::shared::source_parsers::{exceptions, holds_exceptions};
use crate::shared::vcs::{Entry, FileKind, Repository, Staging};
use anyhow::{Context, Result};
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

struct Change<'a> {
    path: &'a str,
    before: Vec<Added>,
    after: Vec<Added>,
}

struct Request<'a> {
    path: &'a str,
    object: &'a str,
    found: &'a mut Vec<Added>,
}

pub(super) fn stageable(
    repository: &Repository<'_>,
    paths: &[String],
) -> Result<BTreeMap<String, Entry>> {
    let (stored, hashed): (Vec<String>, Vec<String>) = paths
        .iter()
        .cloned()
        .partition(|path| holds_exceptions(Path::new(path)));
    repository.stageable(Staging {
        stored: &stored,
        hashed: &hashed,
    })
}
pub(super) fn added(
    repository: &Repository<'_>,
    paths: &[String],
    contents: Contents<'_>,
) -> Result<Vec<Added>> {
    let mut changes: Vec<Change<'_>> = paths
        .iter()
        .filter(|path| holds_exceptions(Path::new(path)))
        .map(|path| Change {
            path,
            before: Vec::new(),
            after: Vec::new(),
        })
        .collect();
    let requests = requests_for(&mut changes, &contents);
    let objects: Vec<&str> = requests.iter().map(|request| request.object).collect();
    let mut pending = requests.into_iter();
    repository.blobs(&objects, move |content| {
        let request = pending
            .next()
            .context("Git returned an unrequested object")?;
        *request.found = keyed(request.path, content)?;
        Ok(())
    })?;
    Ok(changes
        .into_iter()
        .flat_map(|change| beyond(change.after, change.before.iter().map(|found| &found.key)))
        .collect())
}
fn requests_for<'r>(changes: &'r mut [Change<'_>], contents: &Contents<'r>) -> Vec<Request<'r>> {
    let mut requests = Vec::new();
    for change in changes {
        let path = change.path;
        let sides = [
            (contents.committed, &mut change.before),
            (contents.current, &mut change.after),
        ];
        requests.extend(sides.into_iter().filter_map(|(tree, found)| {
            let object = file_object(tree.get(path))?;
            Some(Request {
                path,
                object,
                found,
            })
        }));
    }
    requests
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
fn file_object(entry: Option<&Entry>) -> Option<&str> {
    let entry = entry?;
    match entry.kind {
        FileKind::File | FileKind::Executable => Some(&entry.object),
        FileKind::Symlink | FileKind::Submodule => None,
    }
}
fn keyed(path: &str, content: Vec<u8>) -> Result<Vec<Added>> {
    let Ok(source) = String::from_utf8(content) else {
        return Ok(Vec::new());
    };
    let lines: Vec<&str> = source.lines().collect();
    Ok(exceptions(Path::new(path), &source)?
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
