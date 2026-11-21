// DECISION: D020
// DECISION: D215
use super::{format, invariant_history, revision_memory};
use crate::project_config::Context;
use crate::shared::fs_with_context as fs;
use crate::shared::vcs::Repository;
use anyhow::{Context as _, Result, ensure};
use std::path::Path;

pub fn check(context: &Context, root: &Path, candidate: Option<&str>) -> Result<()> {
    let Some(repository) = crate::shared::vcs::Repository::discover(root)? else {
        return Ok(());
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
    for revision in &revisions {
        check_revision(context, &repository, revision)?;
    }
    match fork {
        Some(fork) => invariant_history::check(context, &repository, &fork),
        None => Ok(()),
    }
}

fn check_revision(context: &Context, repository: &Repository<'_>, revision: &str) -> Result<()> {
    let tree = repository.tree(revision)?;
    let prior_memory = revision_memory::directory(context, repository, revision, &tree)?;
    let index = format!("{prior_memory}/Decisions.md");
    let no_prior_memory = !tree.contains_key(&index);
    if no_prior_memory {
        return Ok(());
    }
    let rows = committed_table(repository, revision, Path::new(&index))?;
    let paths: Vec<String> = rows
        .iter()
        .map(|old| format!("{prior_memory}/{detail}", detail = old.detail))
        .collect();
    let details = revision_memory::read_all(repository, &tree, &paths)?;
    let memory = context.path(&context.config.paths.memory)?;
    let current = format::table(&memory.join("Decisions.md"), format::Catalog::Decisions)?;
    for (old, detail) in rows.into_iter().zip(details) {
        let row = current
            .iter()
            .find(|row| row.id == old.id)
            .with_context(|| format!("{}: committed decision cannot be removed", old.id))?;
        preserve(old, row, &detail, &memory)?;
    }
    Ok(())
}

fn committed_table(
    repository: &Repository<'_>,
    revision: &str,
    path: &Path,
) -> Result<Vec<format::Row>> {
    format::parse_table(
        &revision_memory::read(repository, revision, path)?,
        path,
        format::Catalog::Decisions,
    )
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
