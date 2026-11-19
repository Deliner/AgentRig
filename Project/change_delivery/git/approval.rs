// DECISION: D213
use crate::project_config::Context;
use crate::shared::atomic_files::{atomic_json, overlay::Overlay};
use crate::shared::console;
use crate::shared::vcs::{Entry, FileKind, Repository, Revisions};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const RECORD: &str = "approved";

#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Content {
    Staged { kind: FileKind, object: String },
    Deleted,
}
impl Content {
    fn of(index: &BTreeMap<String, Entry>, path: &str) -> Self {
        index.get(path).map_or(Self::Deleted, |entry| Self::Staged {
            kind: entry.kind,
            object: entry.object.clone(),
        })
    }
}

fn record_path(context: &Context) -> Result<PathBuf> {
    Ok(context.path(&context.config.paths.runtime)?.join(RECORD))
}

pub fn approve(context: &Context) -> Result<i32> {
    let repository = Repository::discover(&context.root)?
        .ok_or_else(|| anyhow::anyhow!("approve requires a VCS repository"))?;
    let mirror = context
        .path(&context.config.paths.runtime)?
        .join("proposal");
    let overlay = Overlay {
        mirror: &mirror,
        root: &context.root,
    };
    let applied = overlay.apply()?;
    let reloaded = match Context::load(&context.root) {
        Ok(reloaded) => reloaded,
        Err(error) => {
            applied.restore()?;
            return Err(error.context(
                "the proposal leaves the configuration invalid; the previous files are restored and the proposal is kept",
            ));
        }
    };
    let approved = record(&reloaded, &repository)?;
    applied.commit()?;
    let none = approved.is_empty();
    if none {
        console::stdout("no change of protected files to approve");
    } else {
        console::stdout(format!(
            "approved; stage and commit: {}",
            approved.join(", ")
        ));
    }
    Ok(0)
}

pub fn record(context: &Context, repository: &Repository<'_>) -> Result<Vec<String>> {
    let status = repository.run(&[
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
        "--no-renames",
    ])?;
    let changed = String::from_utf8(status)?;
    let protected: Vec<String> = changed
        .split('\0')
        .filter_map(|entry| entry.get(3..))
        .filter(|path| context.protection.touched(Path::new(path)))
        .map(str::to_owned)
        .collect();
    let stageable = repository.stageable(&protected)?;
    let approved: BTreeMap<&str, Content> = protected
        .iter()
        .map(|path| (path.as_str(), Content::of(&stageable, path)))
        .collect();
    atomic_json::save(&record_path(context)?, &approved)?;
    Ok(protected)
}

pub(super) fn guard(context: &Context, repository: &Repository<'_>) -> Result<i32> {
    let staged = repository.staged_changes()?;
    let protected: Vec<&String> = staged
        .iter()
        .filter(|path| context.protection.touched(Path::new(path)))
        .collect();
    let unchanged = protected.is_empty();
    if unchanged {
        return Ok(0);
    }
    let index = repository.index()?;
    let committed = repository.tree("HEAD")?;
    let approved: BTreeMap<String, Content> =
        atomic_json::load(&record_path(context)?)?.unwrap_or_default();
    let mut refused = Vec::new();
    for path in protected {
        let pinned = context.protection.pin() == Some(Path::new(path));
        let moved = pinned && committed.contains_key(path.as_str());
        let introduced = pinned && !moved;
        let accepted = if moved {
            forward(repository, &index, path)?
        } else {
            introduced || approved.get(path.as_str()) == Some(&Content::of(&index, path))
        };
        let unapproved = !accepted;
        if unapproved {
            refused.push(path.as_str());
        }
    }
    let accepted = refused.is_empty();
    if accepted {
        return Ok(0);
    }
    refuse(context, &refused);
    Ok(1)
}

fn refuse(context: &Context, refused: &[&str]) {
    let skills = &context.config.paths.skills;
    let skill = context
        .caller
        .skill(&format!("{skills}/propose-protected-change/SKILL.md"));
    console::stderr(format!(
        "AgentRig refused this commit: the user has not approved the staged content of the protected enforcement files {}, or a staged pin does not descend from the committed one. ACTION: Apply {skill}.",
        refused.join(", ")
    ));
}

fn forward(
    repository: &Repository<'_>,
    index: &BTreeMap<String, Entry>,
    path: &str,
) -> Result<bool> {
    let removed = !index.contains_key(path);
    if removed {
        return Ok(false);
    }
    let revision = |source: &str| -> Result<String> {
        let bytes = repository.read(source, Path::new(path))?;
        Ok(String::from_utf8(bytes)?.trim().to_owned())
    };
    let from = revision("HEAD")?;
    let to = revision("")?;
    repository.ancestor(Revisions {
        from: &from,
        to: &to,
    })
}
