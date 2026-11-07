// DECISION: D035
use crate::project_config::Check;
use crate::shared::paths::patterns::globs;
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
        matches!(self, Self::Staged)
    }
}

pub fn changes(root: &Path, index: Index, base: Option<&str>) -> Result<Option<Vec<PathBuf>>> {
    let complete = !index.staged() && base.is_none();
    if complete {
        return Ok(None);
    }
    let repository = crate::shared::vcs::Repository::discover(root)?
        .context("affected test selection requires a repository")?;
    let changes = match base {
        Some(base) => repository.changed_paths(base, "HEAD")?,
        None => repository.staged_changes()?,
    };
    Ok(Some(changes.into_iter().map(PathBuf::from).collect()))
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

pub fn targets(check: &Check, changes: Option<&[PathBuf]>) -> Result<Option<Vec<String>>> {
    let Some(changes) = changes else {
        return Ok(None);
    };
    let unmapped = check.affected.is_empty();
    if unmapped {
        return Ok(None);
    }
    let map = AffectedMap::new(check)?;
    let mut targets = BTreeSet::new();
    for path in changes {
        let Some(selected) = map.targets(path) else {
            return Ok(None);
        };
        targets.extend(selected.into_iter().cloned());
    }
    Ok(Some(targets.into_iter().collect()))
}
