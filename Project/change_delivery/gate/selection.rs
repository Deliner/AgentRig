// DECISION: D035
use crate::project_config::Check;
use crate::shared::paths::patterns::globs;
use crate::shared::vcs::Revisions;
use anyhow::{Context, Result};
use globset::GlobSet;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

// DECISION: D062
#[derive(Clone, Copy)]
pub enum Index {
    Staged,
    Working,
}
impl Index {
    fn staged(self) -> bool {
        match self {
            Self::Staged => true,
            Self::Working => false,
        }
    }
}

// DECISION: D154
pub enum Scope<T> {
    Full,
    Only(Vec<T>),
}

pub fn changes(root: &Path, index: Index, base: Option<&str>) -> Result<Scope<PathBuf>> {
    let complete = !index.staged() && base.is_none();
    if complete {
        return Ok(Scope::Full);
    }
    let repository = crate::shared::vcs::Repository::discover(root)?
        .context("affected test selection requires a repository")?;
    let changes = match base {
        Some(base) => repository.changed_paths(Revisions {
            from: base,
            to: "HEAD",
        })?,
        None => repository.staged_changes()?,
    };
    Ok(Scope::Only(
        changes.into_iter().map(PathBuf::from).collect(),
    ))
}

struct Group<'a> {
    include: GlobSet,
    targets: &'a [String],
}

pub struct AffectedMap<'a> {
    groups: Vec<Group<'a>>,
}

impl<'a> AffectedMap<'a> {
    pub fn new(check: &'a Check) -> Result<Self> {
        let groups = check
            .affected
            .iter()
            .map(|group| {
                Ok(Group {
                    include: globs(&group.include)?,
                    targets: &group.targets,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { groups })
    }
    pub fn targets(&self, path: &Path) -> Option<Vec<&'a String>> {
        let mut matching = self
            .groups
            .iter()
            .filter(|group| group.include.is_match(path))
            .peekable();
        let matched = matching.peek().is_some();
        matched.then(|| matching.flat_map(|group| group.targets).collect())
    }
}

pub fn targets(check: &Check, changes: &Scope<PathBuf>) -> Result<Scope<String>> {
    let changes = match changes {
        Scope::Full => return Ok(Scope::Full),
        Scope::Only(changes) => changes,
    };
    let unmapped = check.affected.is_empty();
    if unmapped {
        return Ok(Scope::Full);
    }
    let map = AffectedMap::new(check)?;
    let mut targets = BTreeSet::new();
    for path in changes {
        let Some(selected) = map.targets(path) else {
            return Ok(Scope::Full);
        };
        targets.extend(selected.into_iter().cloned());
    }
    Ok(Scope::Only(targets.into_iter().collect()))
}
