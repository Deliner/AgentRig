// DECISION: D020
use super::super::config::{Config, Context, FILE};
use super::format;
use anyhow::{Context as _, Result, ensure};
use review_runner::vcs::{Entry, Repository};
use std::{collections::BTreeMap, fs, path::Path};

fn committed(repository: &Repository<'_>, revision: &str, path: &str) -> Result<String> {
    let bytes = repository
        .read(revision, path)
        .with_context(|| format!("cannot read committed memory {path}"))?;
    Ok(String::from_utf8(bytes)?)
}
pub fn check(context: &Context, root: &Path, candidate: Option<&str>) -> Result<()> {
    // A standalone tree or an unborn repository has no committed memory baseline.
    let Some(repository) = Repository::discover(root)? else {
        return Ok(());
    };
    let revisions = match candidate {
        Some(revision) => repository.parents(revision)?,
        None => repository.head()?.into_iter().collect(),
    };
    for revision in revisions {
        check_revision(context, &repository, &revision)?;
    }
    Ok(())
}

fn check_revision(context: &Context, repository: &Repository<'_>, revision: &str) -> Result<()> {
    let tree = repository.tree(revision)?;
    let prior_memory = prior_memory(context, repository, revision, &tree)?;
    let index = format!("{}/Decisions.md", prior_memory);
    let no_prior_memory = !tree.contains_key(&index);
    if no_prior_memory {
        return Ok(());
    }
    let rows = committed_table(repository, revision, &index)?;
    let memory = context.path(&context.config.paths.memory)?;
    let current = format::table(&memory.join("Decisions.md"), 'D')?;
    for old in rows {
        let row = current
            .iter()
            .find(|row| row.id == old.id)
            .with_context(|| format!("{}: committed decision cannot be removed", old.id))?;
        let detail = committed(
            repository,
            revision,
            &format!("{}/{}", prior_memory, old.detail),
        )?;
        preserve(
            old,
            row,
            &detail,
            &fs::read_to_string(memory.join(&row.detail))?,
        )?;
    }
    Ok(())
}

fn committed_table(
    repository: &Repository<'_>,
    revision: &str,
    path: &str,
) -> Result<Vec<format::Row>> {
    format::parse_table(
        &committed(repository, revision, path)?,
        Path::new(path),
        'D',
    )
}

fn prior_memory(
    context: &Context,
    repository: &Repository<'_>,
    revision: &str,
    tree: &BTreeMap<String, Entry>,
) -> Result<String> {
    let current = tree.contains_key(FILE);
    if current {
        let prior: Config =
            review_runner::config::yaml::decode(&committed(repository, revision, FILE)?)
                .context("committed agentrig.yaml schema")?;
        return Ok(prior.paths.memory);
    }
    let legacy = super::super::upgrade::migration::LEGACY_FILE;
    let migrated = tree.contains_key(legacy);
    if migrated {
        return super::super::upgrade::migration::historical_memory(&committed(
            repository, revision, legacy,
        )?);
    }
    Ok(context.config.paths.memory.clone())
}

fn preserve(
    old: format::Row,
    row: &format::Row,
    prior_detail: &str,
    current_detail: &str,
) -> Result<()> {
    ensure!(
        row.detail == old.detail && row.cells[1] == old.cells[1],
        "{}: committed decision identity cannot change",
        old.id
    );
    ensure!(
        current_detail == prior_detail,
        "{}: committed decision detail cannot change; supersede with a new decision",
        old.id
    );
    Ok(())
}
