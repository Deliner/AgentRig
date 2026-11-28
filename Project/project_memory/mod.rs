// DECISION: D019
// DECISION: D016
// DECISION: D013
// DECISION: D009
// DECISION: D008
// DECISION: D002
mod briefing;
mod cards;
mod check_evidence;
mod exceptions;
mod format;
mod glossary;
mod history;
mod invariant_history;
mod oracles;
mod resume;
mod revision_memory;
mod source;
#[cfg(test)]
mod tests;
pub use briefing::{GLOSSARY, NORTH_STAR, briefing};
pub use cards::active_card;
pub use check_evidence::{
    Attempt, Input, native_index, report as report_checks, resume as observed_checks,
};
pub use format::{Catalog, reset_state, state_holds_notes, state_template, table};
pub use resume::run as resume;

use crate::project_config::Context;
use crate::shared::console;
use crate::shared::fs_with_context as fs;
use crate::shared::paths::globs;
use crate::structural_lint;
use anyhow::{Context as _, Result, ensure};
use format::{Row, sections, target};
use source::{Invariant, Sources};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
};

type Applications = HashMap<String, HashSet<PathBuf>>;

pub fn check(context: &Context) -> Result<i32> {
    check_with_history(context, &context.root, None)
}
pub fn check_with_history(context: &Context, root: &Path, revision: Option<&str>) -> Result<i32> {
    let memory = context.path(&context.config.paths.memory)?;
    let mut sources = Sources::default();
    let committed = history::check(context, root, revision, &mut sources)?;
    cards::check(context, &memory)?;
    let applications = decisions(context, &memory, &committed, &mut sources)?;
    let files = structural_lint::maintained_files(&context.root, &globs(&[])?)?;
    references(context, &files, &mut sources, &applications)?;
    exceptions::check(context, &memory, &files)?;
    invariants(context, &memory, &files, &mut sources)?;
    sections(&memory.join("State.md"), format::STATE_SECTIONS, None)?;
    glossary::check(&memory)?;
    Ok(0)
}
// DECISION: D037
// DECISION: D216
fn references(
    context: &Context,
    files: &[PathBuf],
    sources: &mut Sources,
    applications: &Applications,
) -> Result<()> {
    let memory = context.path(&context.config.paths.memory)?;
    let invariants = invariant_ids(&memory)?;
    let selected = globs(&context.config.paths.sources)?;
    let chosen = files.iter().filter(|path| selected.is_match(path));
    sources.prepare(chosen.clone().map(|path| context.root.join(path)))?;
    let mut unlinked = BTreeSet::new();
    for path in chosen {
        let file = context.root.join(path);
        let Some(source) = sources.inspect(&file)? else {
            continue;
        };
        let file = crate::shared::paths::resolve(&file)?;
        let decisions = source.references().filter(|id| !invariants.contains(*id));
        for id in decisions {
            let linked = applications.get(id).with_context(|| {
                format!(
                    "{}: {id} is not a recorded decision or invariant",
                    path.display()
                )
            })?;
            let outside = !linked.contains(&file);
            if outside {
                unlinked.insert(format!("{id} in {}", path.display()));
            }
        }
    }
    let unlinked: Vec<String> = unlinked.into_iter().collect();
    ensure!(
        unlinked.is_empty(),
        "a decision marker counts only in a file its decision links under Applies in; link the file from the decision or remove the marker: {}",
        unlinked.join(", ")
    );
    Ok(())
}
fn invariant_ids(memory: &Path) -> Result<HashSet<String>> {
    Ok(table(&memory.join("Invariants.md"), Catalog::Invariants)?
        .into_iter()
        .map(|row| row.id)
        .collect())
}
fn details(context: &Context, memory: &Path, rows: &[Row], catalog: Catalog) -> Result<()> {
    let layout = catalog.detail_sections();
    for row in rows {
        let path = detail(context, memory, row, catalog)?;
        sections(&path, layout.required, layout.optional)?;
    }
    reject_unindexed(memory, catalog, &indexed(rows))
}
fn detail(context: &Context, memory: &Path, row: &Row, catalog: Catalog) -> Result<PathBuf> {
    let directory = catalog.directory();
    ensure!(
        row.id
            .get(1..)
            .is_some_and(|number| row.detail == format!("{directory}/{number}.md")),
        "{}: detail path does not match ID",
        row.id
    );
    target(context, memory, &row.detail)
}
fn indexed(rows: &[Row]) -> HashSet<String> {
    rows.iter().map(|row| row.detail.clone()).collect()
}
// DECISION: D236
fn decision_details(
    context: &Context,
    memory: &Path,
    rows: &[Row],
    committed: &HashSet<String>,
) -> Result<()> {
    let layout = Catalog::Decisions.detail_sections();
    for row in rows {
        let path = detail(context, memory, row, Catalog::Decisions)?;
        let prior_layout = committed.contains(&row.id)
            && sections(&path, format::PRIOR_DECISION_SECTIONS, None).is_ok();
        if prior_layout {
            continue;
        }
        sections(&path, layout.required, layout.optional)?;
    }
    reject_unindexed(memory, Catalog::Decisions, &indexed(rows))
}
fn reject_unindexed(memory: &Path, catalog: Catalog, known: &HashSet<String>) -> Result<()> {
    let directory = catalog.directory();
    let Some(entries) = fs::present(fs::read_dir(memory.join(directory)))? else {
        return Ok(());
    };
    for entry in entries {
        let path = entry?.path();
        let numeric = path
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()));
        if numeric {
            let name = path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("detail path has no file name"))?
                .to_string_lossy();
            ensure!(
                known.contains(&format!("{directory}/{name}")),
                "unindexed detail {}",
                path.display()
            );
        }
    }
    Ok(())
}
fn decisions(
    context: &Context,
    memory: &Path,
    committed: &HashSet<String>,
    sources: &mut Sources,
) -> Result<Applications> {
    let rows = table(&memory.join("Decisions.md"), Catalog::Decisions)?;
    decision_details(context, memory, &rows, committed)?;
    let linked: Vec<(&Row, Vec<Result<PathBuf>>)> = rows
        .iter()
        .map(|row| {
            let links = format::links(&row.cells[2]);
            let paths = links.iter().map(|link| target(context, memory, link));
            (row, paths.collect())
        })
        .collect();
    let mut prepared = Vec::new();
    for path in linked.iter().flat_map(|(_, paths)| paths).flatten() {
        prepared.extend(marked_source(path).then(|| path.clone()));
    }
    sources.prepare(prepared)?;
    let mut applications = HashMap::new();
    for (row, paths) in linked {
        applications.insert(row.id.clone(), markers(row, paths, sources)?);
    }
    Ok(applications)
}
fn markers(
    row: &Row,
    paths: Vec<Result<PathBuf>>,
    sources: &mut Sources,
) -> Result<HashSet<PathBuf>> {
    ensure!(!paths.is_empty(), "{}: application link required", row.id);
    let mut applied = HashSet::new();
    for path in paths {
        let path = path?;
        marker(row, &path, sources)?;
        applied.insert(path);
    }
    Ok(applied)
}
fn marker(row: &Row, path: &Path, sources: &mut Sources) -> Result<()> {
    let markers_unchecked = !marked_source(path);
    if markers_unchecked {
        return Ok(());
    }
    match sources.inspect(path)? {
        Some(source) => ensure!(
            source.marker(&row.id),
            "{}: decision marker absent in {}",
            row.id,
            path.display()
        ),
        None => console::stderr(format!(
            "WARNING [{id}]: marker inspection unsupported for {path}",
            id = row.id,
            path = path.display()
        )),
    }
    Ok(())
}
fn marked_source(path: &Path) -> bool {
    let non_source = path.extension().is_some_and(|ext| {
        ["md", "toml", "json", "yaml", "yml", "just"]
            .iter()
            .any(|kind| ext == *kind)
    }) || path.file_name().is_some_and(|name| name == "justfile");
    !non_source
}
fn invariants(
    context: &Context,
    memory: &Path,
    files: &[PathBuf],
    sources: &mut Sources,
) -> Result<()> {
    let rows = table(&memory.join("Invariants.md"), Catalog::Invariants)?;
    details(context, memory, &rows, Catalog::Invariants)?;
    let mut crates = oracles::CrateModules::new(files);
    let mut discoveries = Vec::new();
    for row in &rows {
        let oracle = format::link(&row.cells[2])?;
        let path = target(context, memory, &oracle.path)?;
        let source = sources
            .inspect(&path)?
            .ok_or_else(|| anyhow::anyhow!("{}: unsupported oracle source language", row.id))?;
        let id = &row.id;
        let linked = oracles::Linked {
            path: &path,
            source,
        };
        ensure!(
            oracles::marked(context, id, linked, &mut crates)?,
            "{}: marked oracle function {} missing or ambiguous",
            row.id,
            oracle.name
        );
        let function = &oracle.name;
        let invariant = Invariant { id, function };
        discoveries.push(oracles::validate(context, invariant, &path)?);
    }
    oracles::discover(context, &discoveries)
}
