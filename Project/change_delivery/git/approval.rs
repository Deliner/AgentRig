// DECISION: D213
use super::exceptions::{self, Added, Contents, Key};
use crate::project_config::{Context, protection::Protection};
use crate::shared::atomic_files::{Overlay, load_json, save_json};
use crate::shared::console;
use crate::shared::vcs::{Entry, FileKind, Repository, Revisions};
use crate::structural_lint;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const RECORD: &str = "approved";

// DECISION: D220
#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct Record {
    files: BTreeMap<String, Content>,
    exceptions: Vec<Key>,
}
pub struct Approved {
    files: Vec<String>,
    exceptions: Vec<String>,
}

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
    let mirror = super::proposal::mirror(context)?;
    let overlay = Overlay {
        mirror: &mirror,
        root: &context.root,
    };
    // DECISION: D218
    super::pin::install(context, &repository, &overlay)?;
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
    approved.report();
    Ok(0)
}

impl Approved {
    fn report(&self) {
        let Self { files, exceptions } = self;
        let none = files.is_empty() && exceptions.is_empty();
        if none {
            console::stdout("no change of protected files and no new lint exception to approve");
        }
        let changed = !files.is_empty();
        if changed {
            console::stdout(format!("approved; stage and commit: {}", files.join(", ")));
        }
        let excepted = !exceptions.is_empty();
        if excepted {
            console::stdout(format!(
                "approved new lint exceptions; stage and commit:\n{}",
                exceptions.join("\n")
            ));
        }
    }
}

pub fn record(context: &Context, repository: &Repository<'_>) -> Result<Approved> {
    let status = repository.run(&[
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
        "--no-renames",
    ])?;
    let changed: Vec<String> = String::from_utf8(status)?
        .split('\0')
        .filter_map(|entry| entry.get(3..))
        .map(str::to_owned)
        .collect();
    let stageable = repository.stageable(&changed)?;
    let committed = committed(repository)?;
    let contents = Contents {
        current: &stageable,
        committed: &committed,
    };
    let added = exceptions::added(repository, &changed, contents)?;
    let files: Vec<String> = changed
        .into_iter()
        .filter(|path| context.protection.touched(Path::new(path)))
        .collect();
    let record = Record {
        files: files
            .iter()
            .map(|path| (path.clone(), Content::of(&stageable, path)))
            .collect(),
        exceptions: added.iter().map(|found| found.key.clone()).collect(),
    };
    save_json(&record_path(context)?, &record)?;
    Ok(Approved {
        files,
        exceptions: added.iter().map(Added::describe).collect(),
    })
}

// DECISION: D226
pub(super) fn admit(context: &Context, repository: &Repository<'_>, path: &Path) -> Result<()> {
    let name = path.to_string_lossy().into_owned();
    let stageable = repository.stageable(std::slice::from_ref(&name))?;
    let location = record_path(context)?;
    let mut record: Record = load_json(&location)?.unwrap_or_default();
    let content = Content::of(&stageable, &name);
    record.files.insert(name, content);
    save_json(&location, &record)
}

fn committed(repository: &Repository<'_>) -> Result<BTreeMap<String, Entry>> {
    match repository.head()? {
        Some(head) => repository.tree(&head),
        None => Ok(BTreeMap::new()),
    }
}

pub(super) fn guard(context: &Context, repository: &Repository<'_>) -> Result<i32> {
    let staged = repository.staged_changes()?;
    let index = repository.index()?;
    let committed = committed(repository)?;
    let contents = Contents {
        current: &index,
        committed: &committed,
    };
    let added = exceptions::added(repository, &staged, contents)?;
    let protected: Vec<&String> = staged
        .iter()
        .filter(|path| context.protection.touched(Path::new(path)))
        .collect();
    let unchanged = protected.is_empty() && added.is_empty();
    if unchanged {
        return Ok(0);
    }
    let record: Record = load_json(&record_path(context)?)?.unwrap_or_default();
    let review = Review {
        repository,
        index,
        committed,
        approved: record.files,
    };
    let refused = review.refused(&context.protection, protected)?;
    let exceptions: Vec<String> = exceptions::unapproved(added, &record.exceptions)
        .iter()
        .map(Added::describe)
        .collect();
    let accepted = refused.is_empty() && exceptions.is_empty();
    if accepted {
        return Ok(0);
    }
    refuse(context, &refused, &exceptions);
    Ok(1)
}

struct Review<'a, 'r> {
    repository: &'a Repository<'r>,
    index: BTreeMap<String, Entry>,
    committed: BTreeMap<String, Entry>,
    approved: BTreeMap<String, Content>,
}
impl Review<'_, '_> {
    fn refused<'p>(&self, protection: &Protection, paths: Vec<&'p String>) -> Result<Vec<&'p str>> {
        let mut refused = Vec::new();
        for path in paths {
            let unapproved = !self.accepted(protection, path)?;
            if unapproved {
                refused.push(path.as_str());
            }
        }
        Ok(refused)
    }
    fn accepted(&self, protection: &Protection, path: &str) -> Result<bool> {
        let file = Path::new(path);
        let approved = self.approved.get(path) == Some(&Content::of(&self.index, path));
        // DECISION: D218
        let pinned = protection.pin() == Some(file);
        if pinned {
            let replaces = approved && self.committed.contains_key(path);
            return if replaces {
                forward(self.repository, &self.index, path)
            } else {
                Ok(approved)
            };
        }
        // DECISION: D226
        let extending = protection.lint() == file && !approved;
        if extending {
            let kept = self.committed.contains_key(path) && self.index.contains_key(path);
            return Ok(kept && structural_lint::only_adds_rules(self.repository, file)?);
        }
        // DECISION: D214
        let lowerable = protection.baseline() == file && !approved;
        if lowerable {
            let removed = !self.index.contains_key(path);
            return Ok(removed || structural_lint::only_lowers_counts(self.repository, file)?);
        }
        Ok(approved)
    }
}

fn refuse(context: &Context, refused: &[&str], exceptions: &[String]) {
    let skill = context.protection_skill();
    let mut reasons = Vec::new();
    let files = !refused.is_empty();
    if files {
        reasons.push(format!(
            "the user has not approved the staged content of the protected enforcement files {}, or an approved pin does not descend from the committed one; without approval a staged lint baseline may only lower the committed counts and a staged lint configuration may only add rules",
            refused.join(", ")
        ));
    }
    // DECISION: D220
    let excepted = !exceptions.is_empty();
    if excepted {
        reasons.push(format!(
            "the user has not approved these new lint exceptions, each shown as file:line, decision, rule and the line it lifts:\n{}\n",
            exceptions.join("\n")
        ));
    }
    console::stderr(format!(
        "AgentRig refused this commit: {}. ACTION: Apply {skill}.",
        reasons.join("; ")
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
