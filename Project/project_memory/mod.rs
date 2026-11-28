// DECISION: D019
// DECISION: D016
// DECISION: D013
// DECISION: D012
// DECISION: D009
// DECISION: D008
// DECISION: D002
mod cards;
mod check_evidence;
mod exceptions;
mod format;
mod history;
mod invariant_history;
mod oracles;
mod resume;
mod revision_memory;
mod source;
#[cfg(test)]
mod tests;
pub use cards::{active as active_card, planned as planned_cards};
pub use check_evidence::{
    Attempt, Input, native_index, report as report_checks, resume as observed_checks,
};
pub use format::{Catalog, reset_state, state_holds_notes, state_template};
pub use resume::run as resume;

use crate::project_config::Context;
use crate::shared::console;
use crate::shared::fs_with_context as fs;
use crate::shared::paths::globs;
use crate::structural_lint;
use anyhow::{Context as _, Result, ensure};
use format::{Row, sections, table, target};
use source::{Invariant, Sources};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
};

type Applications = HashMap<String, HashSet<PathBuf>>;

pub fn check(context: &Context) -> Result<i32> {
    check_with_history(context, &context.root, None)
}
pub fn check_with_history(context: &Context, root: &Path, revision: Option<&str>) -> Result<i32> {
    let memory = context.path(&context.config.paths.memory)?;
    let mut sources = Sources::default();
    history::check(context, root, revision, &mut sources)?;
    plan(context, &memory)?;
    cards::check(context, &memory)?;
    backlog(context, &memory)?;
    let applications = decisions(context, &memory, &mut sources)?;
    let files = structural_lint::maintained_files(&context.root, &globs(&[])?)?;
    references(context, &files, &mut sources, &applications)?;
    exceptions::check(context, &memory, &files)?;
    invariants(context, &memory, &mut sources)?;
    sections(&memory.join("State.md"), format::STATE_SECTIONS, None)?;
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
    let directory = catalog.directory();
    let mut known = HashSet::new();
    for row in rows {
        ensure!(
            row.id
                .get(1..)
                .is_some_and(|number| row.detail == format!("{directory}/{number}.md")),
            "{}: detail path does not match ID",
            row.id
        );
        known.insert(row.detail.clone());
        sections(
            &target(context, memory, &row.detail)?,
            layout.required,
            layout.optional,
        )?;
    }
    reject_unindexed(memory, catalog, &known)
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
// DECISION: D050
fn backlog(context: &Context, memory: &Path) -> Result<()> {
    let mut ids = HashSet::new();
    for root in [memory.to_path_buf(), memory.join("Archive")] {
        let path = root.join("Backlog.md");
        let adopted = fs::try_exists(&path)?;
        if adopted {
            let rows = table(&path, Catalog::Backlog)?;
            details(context, &root, &rows, Catalog::Backlog)?;
            for row in rows {
                ensure!(
                    ids.insert(row.id),
                    "duplicate ID across Backlog and Archive"
                );
            }
        } else {
            reject_unindexed(&root, Catalog::Backlog, &HashSet::new())?;
        }
    }
    Ok(())
}
// DECISION: D127
// DECISION: D031
fn plan(context: &Context, memory: &Path) -> Result<()> {
    let mut rows = plan_rows(context, memory)?;
    let archive = memory.join("Archive");
    let archived = fs::try_exists(archive.join("Plan.md"))?;
    if archived {
        let completed = plan_rows(context, &archive)?;
        ensure!(
            completed.iter().all(|row| row.cells[1] == "complete"),
            "Archive requires completed Plan outcomes"
        );
        rows.extend(completed);
    } else {
        reject_unindexed(&archive, Catalog::Plan, &HashSet::new())?;
    }
    ensure!(
        rows.iter().filter(|row| row.cells[1] == "active").count() <= 1,
        "Plan has multiple active features"
    );
    let index: HashMap<_, _> = rows.iter().map(|row| (row.id.as_str(), row)).collect();
    ensure!(
        index.len() == rows.len(),
        "duplicate ID across Plan and Archive"
    );
    let mut graph = BTreeMap::new();
    for row in &rows {
        graph.insert(row.id.as_str(), dependencies(row, &index)?);
    }
    cards::acyclic(&graph)
}
fn plan_rows(context: &Context, memory: &Path) -> Result<Vec<Row>> {
    let rows = table(&memory.join("Plan.md"), Catalog::Plan)?;
    details(context, memory, &rows, Catalog::Plan)?;
    for row in &rows {
        plan_status(row, memory)?;
    }
    Ok(rows)
}
fn plan_status(row: &Row, memory: &Path) -> Result<()> {
    let status = row.cells[1].as_str();
    ensure!(
        ["pending", "active", "paused", "complete"].contains(&status),
        "{}: invalid status {status}",
        row.id
    );
    let delivery_required = ["paused", "complete"].contains(&status);
    if delivery_required {
        let source = fs::read_to_string(memory.join(&row.detail))?;
        ensure!(
            source.lines().any(|line| line == "## Delivery"),
            "{}: {status} requires Delivery",
            row.id
        );
    }
    Ok(())
}
fn dependencies<'a>(row: &'a Row, index: &HashMap<&str, &Row>) -> Result<Vec<&'a str>> {
    let independent = row.cells[2] == "-";
    let dependencies: Vec<_> = if independent {
        Vec::new()
    } else {
        row.cells[2].split(',').map(str::trim).collect()
    };
    ensure!(
        dependencies.iter().collect::<HashSet<_>>().len() == dependencies.len(),
        "{}: duplicate dependencies",
        row.id
    );
    let requires_completed = ["active", "complete"].contains(&row.cells[1].as_str());
    for dependency in &dependencies {
        let prior = index
            .get(dependency)
            .ok_or_else(|| anyhow::anyhow!("{}: unknown dependency {dependency}", row.id))?;
        ensure!(
            !requires_completed || prior.cells[1] == "complete",
            "{}: prerequisite {dependency} is incomplete",
            row.id
        );
    }
    Ok(dependencies)
}
fn decisions(context: &Context, memory: &Path, sources: &mut Sources) -> Result<Applications> {
    let rows = table(&memory.join("Decisions.md"), Catalog::Decisions)?;
    details(context, memory, &rows, Catalog::Decisions)?;
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
fn invariants(context: &Context, memory: &Path, sources: &mut Sources) -> Result<()> {
    let rows = table(&memory.join("Invariants.md"), Catalog::Invariants)?;
    details(context, memory, &rows, Catalog::Invariants)?;
    let mut discoveries = Vec::new();
    for row in &rows {
        let oracle = format::link(&row.cells[2])?;
        let path = target(context, memory, &oracle.path)?;
        let source = sources
            .inspect(&path)?
            .ok_or_else(|| anyhow::anyhow!("{}: unsupported oracle source language", row.id))?;
        let id = &row.id;
        ensure!(
            oracles::marked(context, id, source)?,
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
