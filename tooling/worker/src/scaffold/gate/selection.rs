// DECISION: D035
use super::super::config::Check;
use crate::lint::config::globs;
use anyhow::{Context, Result};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub fn changes(root: &Path, staged: bool, base: Option<&str>) -> Result<Option<Vec<PathBuf>>> {
    let complete = !staged && base.is_none();
    if complete {
        return Ok(None);
    }
    let repository = crate::vcs::Repository::discover(root)?
        .context("affected test selection requires a repository")?;
    let changes = match base {
        Some(base) => repository.changed_paths(base, "HEAD")?,
        None => repository.staged_changes()?,
    };
    Ok(Some(changes.into_iter().map(PathBuf::from).collect()))
}

pub fn targets(check: &Check, changes: Option<&[PathBuf]>) -> Result<Option<Vec<String>>> {
    let Some(changes) = changes else {
        return Ok(None);
    };
    let unmapped = check.affected.is_empty();
    if unmapped {
        return Ok(None);
    }
    let groups = check
        .affected
        .iter()
        .map(|group| Ok((globs(&group.include)?, &group.targets)))
        .collect::<Result<Vec<_>>>()?;
    let mut targets = BTreeSet::new();
    for path in changes {
        let mut covered = false;
        for (include, selected) in &groups {
            let matches = include.is_match(path);
            if matches {
                covered = true;
                targets.extend(selected.iter().cloned());
            }
        }
        let unknown = !covered;
        if unknown {
            return Ok(None);
        }
    }
    Ok(Some(targets.into_iter().collect()))
}
