// DECISION: D215
use super::{format, history, source};
use crate::project_config::{Context, Oracle, module};
use crate::shared::fs_with_context as fs;
use crate::shared::vcs::{Entry, Repository};
use anyhow::{Result, ensure};
use format::{Catalog, Row};
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};

struct Prior<'a> {
    repository: &'a Repository<'a>,
    revision: &'a str,
    tree: BTreeMap<String, Entry>,
    memory: String,
    oracles: BTreeMap<String, Oracle>,
}

#[derive(Clone, Copy)]
struct Change<'a> {
    old: &'a Row,
    row: &'a Row,
}

pub fn check(context: &Context, repository: &Repository<'_>, fork: &str) -> Result<()> {
    let tree = repository.tree(fork)?;
    let memory = history::prior_memory(context, repository, fork, &tree)?;
    let mut prior = Prior {
        repository,
        revision: fork,
        tree,
        memory,
        oracles: BTreeMap::new(),
    };
    if let Some(text) = prior.text(&module("oracles"))? {
        prior.oracles = crate::shared::strict_yaml::decode(&text)?;
    }
    let Some(rows) = prior.table("Invariants.md", Catalog::Invariants)? else {
        return Ok(());
    };
    let current_memory = context.path(&context.config.paths.memory)?;
    let current = format::table(&current_memory.join("Invariants.md"), Catalog::Invariants)?;
    let introduced = introduced(&prior, &current_memory)?;
    for old in &rows {
        let kept = match current.iter().find(|row| row.id == old.id) {
            Some(row) => unchanged(context, &prior, Change { old, row })?,
            None => false,
        };
        ensure!(
            kept || introduced.contains(&old.id),
            "{id}: the invariant's predicate, oracle or linked test changed since the fork from {base}; add a decision that names {id}",
            id = old.id,
            base = context.config.git.base
        );
    }
    Ok(())
}

fn introduced(prior: &Prior<'_>, memory: &Path) -> Result<HashSet<String>> {
    let decided: HashSet<String> = prior
        .table("Decisions.md", Catalog::Decisions)?
        .unwrap_or_default()
        .into_iter()
        .map(|row| row.id)
        .collect();
    let rows = format::table(&memory.join("Decisions.md"), Catalog::Decisions)?;
    Ok(rows
        .iter()
        .filter(|row| !decided.contains(&row.id))
        .flat_map(|row| row.cells[1].split(|c: char| !c.is_ascii_alphanumeric()))
        .map(str::to_owned)
        .collect())
}

fn unchanged(context: &Context, prior: &Prior<'_>, change: Change<'_>) -> Result<bool> {
    let Change { old, row } = change;
    let id = &old.id;
    let memory = context.path(&context.config.paths.memory)?;
    let statement = row.cells[1] == old.cells[1];
    let prior_detail = prior.text(&prior.located(&old.detail)?)?;
    let detail = prior_detail == Some(fs::read_to_string(memory.join(&row.detail))?);
    let oracle = prior.oracles.get(id) == context.config.oracles.get(id);
    let link = format::link(&row.cells[2])?;
    let linked = format::target(context, &memory, &link.path)?;
    let current = source::inspect(&linked)?.and_then(|mut source| source.marked.remove(id));
    let test = prior.marked(old)? == current;
    Ok([statement, detail, oracle, test]
        .into_iter()
        .all(|same| same))
}

impl Prior<'_> {
    fn text(&self, path: &str) -> Result<Option<String>> {
        let absent = !self.tree.contains_key(path);
        if absent {
            return Ok(None);
        }
        history::committed(self.repository, self.revision, Path::new(path)).map(Some)
    }

    fn table(&self, name: &str, catalog: Catalog) -> Result<Option<Vec<Row>>> {
        let path = format!("{}/{name}", self.memory);
        self.text(&path)?
            .map(|text| format::parse_table(&text, Path::new(&path), catalog))
            .transpose()
    }

    fn marked(&self, old: &Row) -> Result<Option<Vec<String>>> {
        let path = self.located(&format::link(&old.cells[2])?.path)?;
        let Some(text) = self.text(&path)? else {
            return Ok(None);
        };
        let source = source::parse(Path::new(&path), &text)?;
        Ok(source.and_then(|mut source| source.marked.remove(&old.id)))
    }

    fn located(&self, link: &str) -> Result<String> {
        let joined = format!("{}/{link}", self.memory);
        let mut parts: Vec<&str> = Vec::new();
        for part in joined.split('/') {
            match part {
                "" | "." => {}
                ".." => ensure!(
                    parts.pop().is_some(),
                    "memory link leaves the repository: {link}"
                ),
                _ => parts.push(part),
            }
        }
        Ok(parts.join("/"))
    }
}
