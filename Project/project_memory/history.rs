// DECISION: D020
// DECISION: D215
// DECISION: D236
use super::{format, invariant_history, revision_memory::Snapshot, source::Sources};
use crate::project_config::Context;
use crate::shared::fs_with_context as fs;
use anyhow::{Context as _, Result, ensure};
use std::{collections::HashSet, path::Path};

pub fn check(
    context: &Context,
    root: &Path,
    candidate: Option<&str>,
    sources: &mut Sources,
) -> Result<HashSet<String>> {
    let Some(repository) = crate::shared::vcs::Repository::discover(root)? else {
        return Ok(HashSet::new());
    };
    let (mut revisions, tip) = match candidate {
        Some(revision) => (repository.parents(revision)?, Some(revision.to_owned())),
        None => {
            let head = repository.head()?;
            (head.iter().cloned().collect(), head)
        }
    };
    let fork = match tip {
        Some(tip) => repository.fork(&context.config.git, &tip)?,
        None => None,
    };
    let unlisted = fork
        .as_ref()
        .filter(|fork| !revisions.contains(fork))
        .cloned();
    revisions.extend(unlisted);
    let mut committed = HashSet::new();
    let mut at_fork = None;
    for revision in &revisions {
        let mut prior = Snapshot::load(context, &repository, revision)?;
        committed.extend(check_revision(context, &mut prior)?);
        let forked = fork.as_ref().is_some_and(|fork| fork == revision);
        if forked {
            at_fork = Some(prior);
        }
    }
    if let Some(prior) = at_fork {
        invariant_history::check(context, prior, sources)?;
    }
    Ok(committed)
}

fn check_revision(context: &Context, prior: &mut Snapshot<'_>) -> Result<Vec<String>> {
    let Some(rows) = prior.table("Decisions.md", format::Catalog::Decisions)? else {
        return Ok(Vec::new());
    };
    let paths: Vec<String> = rows.iter().map(|old| prior.path(&old.detail)).collect();
    prior.read_ahead(&paths)?;
    let mut details = Vec::new();
    for path in &paths {
        let detail = prior.text(path)?;
        details.push(detail.with_context(|| format!("cannot read committed memory {path}"))?);
    }
    let memory = context.path(&context.config.paths.memory)?;
    let current = format::table(&memory.join("Decisions.md"), format::Catalog::Decisions)?;
    let mut kept = Vec::new();
    for (old, detail) in rows.into_iter().zip(details) {
        let row = current
            .iter()
            .find(|row| row.id == old.id)
            .with_context(|| format!("{}: committed decision cannot be removed", old.id))?;
        kept.push(old.id.clone());
        preserve(old, row, &detail, &memory)?;
    }
    Ok(kept)
}

fn preserve(old: format::Row, row: &format::Row, prior_detail: &str, memory: &Path) -> Result<()> {
    ensure!(
        row.detail == old.detail && row.cells[1] == old.cells[1],
        "{}: committed decision identity cannot change",
        old.id
    );
    let current_detail = fs::read_to_string(memory.join(&row.detail))?;
    ensure!(
        current_detail == prior_detail,
        "{}: committed decision detail cannot change; supersede with a new decision",
        old.id
    );
    Ok(())
}
