// DECISION: D215
use super::{
    format,
    revision_memory::Snapshot,
    source::{self, Sources},
};
use crate::project_config::{Context, Oracle, module};
use crate::shared::fs_with_context as fs;
use anyhow::{Result, ensure};
use format::{Catalog, Row};
use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};

struct Prior<'a> {
    snapshot: Snapshot<'a>,
    oracles: BTreeMap<String, Oracle>,
}

#[derive(Clone, Copy)]
struct Change<'a> {
    old: &'a Row,
    row: &'a Row,
}

pub fn check(context: &Context, mut snapshot: Snapshot<'_>, sources: &mut Sources) -> Result<()> {
    let oracles = match snapshot.text(&module("oracles"))? {
        Some(text) => crate::shared::strict_yaml::decode(&text)?,
        None => BTreeMap::new(),
    };
    let mut prior = Prior { snapshot, oracles };
    let Some(rows) = prior.snapshot.table("Invariants.md", Catalog::Invariants)? else {
        return Ok(());
    };
    prior.read_ahead(&rows)?;
    let current_memory = context.path(&context.config.paths.memory)?;
    let current = format::table(&current_memory.join("Invariants.md"), Catalog::Invariants)?;
    let introduced = introduced(&mut prior.snapshot, &current_memory)?;
    let mut linked = Vec::new();
    for row in &current {
        linked.extend(linked_test(context, &current_memory, row).ok());
    }
    sources.prepare(linked)?;
    for old in &rows {
        let kept = match current.iter().find(|row| row.id == old.id) {
            Some(row) => unchanged(context, &mut prior, Change { old, row }, sources)?,
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

fn introduced(prior: &mut Snapshot<'_>, memory: &Path) -> Result<HashSet<String>> {
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

fn unchanged(
    context: &Context,
    prior: &mut Prior<'_>,
    change: Change<'_>,
    sources: &mut Sources,
) -> Result<bool> {
    let Change { old, row } = change;
    let id = &old.id;
    let memory = context.path(&context.config.paths.memory)?;
    let statement = row.cells[1] == old.cells[1];
    let located = prior.snapshot.located(&old.detail)?;
    let prior_detail = prior.snapshot.text(&located)?;
    let detail = prior_detail == Some(fs::read_to_string(memory.join(&row.detail))?);
    let oracle = prior.oracles.get(id) == context.config.oracles.get(id);
    let linked = linked_test(context, &memory, row)?;
    let current = sources
        .inspect(&linked)?
        .and_then(|source| source.marked.get(id).cloned());
    let test = prior.same_test(old, &linked)? || prior.marked(old)? == current;
    Ok([statement, detail, oracle, test]
        .into_iter()
        .all(|same| same))
}

fn linked_test(context: &Context, memory: &Path, row: &Row) -> Result<PathBuf> {
    let link = format::link(&row.cells[2])?;
    format::target(context, memory, &link.path)
}

impl Prior<'_> {
    fn read_ahead(&mut self, rows: &[Row]) -> Result<()> {
        let mut paths = Vec::new();
        for old in rows {
            paths.extend(self.snapshot.located(&old.detail).ok());
            let link = format::link(&old.cells[2]).ok();
            paths.extend(link.and_then(|link| self.snapshot.located(&link.path).ok()));
        }
        self.snapshot.read_ahead(&paths)
    }

    fn same_test(&mut self, old: &Row, current: &Path) -> Result<bool> {
        let path = self.snapshot.located(&format::link(&old.cells[2])?.path)?;
        let Some(text) = self.snapshot.text(&path)? else {
            return Ok(false);
        };
        let same_name = Path::new(&path).file_name() == current.file_name();
        Ok(same_name && fs::read(current).is_ok_and(|bytes| bytes == text.as_bytes()))
    }

    fn marked(&mut self, old: &Row) -> Result<Option<Vec<String>>> {
        let path = self.snapshot.located(&format::link(&old.cells[2])?.path)?;
        let Some(text) = self.snapshot.text(&path)? else {
            return Ok(None);
        };
        let source = source::parse(Path::new(&path), &text)?;
        Ok(source.and_then(|mut source| source.marked.remove(&old.id)))
    }
}
