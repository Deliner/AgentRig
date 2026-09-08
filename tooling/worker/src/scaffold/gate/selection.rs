use super::super::config::Check;
use crate::lint::config::globs;
use anyhow::{Context, Result};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub fn changes(root: &Path, staged: bool) -> Result<Option<Vec<PathBuf>>> {
    if staged {
        let repository = review_runner::vcs::Repository::discover(root)?
            .context("staged test selection requires a repository")?;
        return Ok(Some(
            repository
                .staged_changes()?
                .into_iter()
                .map(PathBuf::from)
                .collect(),
        ));
    }
    Ok(None)
}

/// None retains the full command; an empty selection skips its test targets.
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
