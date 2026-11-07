// DECISION: D019
// DECISION: D016
// DECISION: D013
// DECISION: D012
// DECISION: D009
// DECISION: D008
// DECISION: D002
pub(crate) mod evidence;
pub(super) mod format;
mod history;
mod oracles;
mod resume;
mod source;
pub use resume::run as resume;

use crate::project_config::Context;
use crate::shared::console;
use crate::shared::fs;
use crate::shared::paths::patterns::globs;
use crate::structural_lint::inventory;
use anyhow::{Result, ensure};
use format::{Catalog, Row, sections, table, target};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::Path,
};

pub fn check(context: &Context) -> Result<i32> {
    check_with_history(context, &context.root, None)
}
pub fn check_with_history(context: &Context, root: &Path, revision: Option<&str>) -> Result<i32> {
    let memory = context.path(&context.config.paths.memory)?;
    history::check(context, root, revision)?;
    plan(context, &memory)?;
    backlog(context, &memory)?;
    decisions(context, &memory)?;
    references(context, &memory)?;
    invariants(context, &memory)?;
    sections(&memory.join("State.md"), format::STATE_SECTIONS, None)?;
    Ok(0)
}
// DECISION: D037
fn references(context: &Context, memory: &Path) -> Result<()> {
    let records: HashSet<String> = table(&memory.join("Decisions.md"), Catalog::Decisions)?
        .into_iter()
        .chain(table(&memory.join("Invariants.md"), Catalog::Invariants)?)
        .map(|row| row.id)
        .collect();
    let sources = globs(&context.config.paths.sources)?;
    let files = inventory::collect(&context.root, &globs(&[])?)?.files;
    for path in files.iter().filter(|path| sources.is_match(path)) {
        let Some(source) = source::inspect(&context.root.join(path))? else {
            continue;
        };
        for id in source.references() {
            ensure!(
                records.contains(id),
                "{}: {id} is not a recorded decision or invariant",
                path.display()
            );
        }
    }
    Ok(())
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
            &target(&context.root, memory, &row.detail)?,
            layout.required,
            layout.optional,
        )?;
    }
    reject_unindexed(memory, catalog, &known)
}
fn reject_unindexed(memory: &Path, catalog: Catalog, known: &HashSet<String>) -> Result<()> {
    let directory = catalog.directory();
    let path = memory.join(directory);
    let directory_exists = path.is_dir();
    if directory_exists {
        for entry in fs::read_dir(path)? {
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
    acyclic(&graph)
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
// DECISION: D127
fn acyclic(graph: &BTreeMap<&str, Vec<&str>>) -> Result<()> {
    let mut done = HashSet::new();
    while done.len() < graph.len() {
        let before = done.len();
        for (id, dependencies) in graph {
            let ready = dependencies.iter().all(|id| done.contains(id));
            if ready {
                done.insert(*id);
            }
        }
        ensure!(done.len() > before, "Plan dependency cycle");
    }
    Ok(())
}
fn decisions(context: &Context, memory: &Path) -> Result<()> {
    let rows = table(&memory.join("Decisions.md"), Catalog::Decisions)?;
    details(context, memory, &rows, Catalog::Decisions)?;
    for row in &rows {
        let links = format::links(&row.cells[2]);
        ensure!(!links.is_empty(), "{}: application link required", row.id);
        for link in links {
            let path = target(&context.root, memory, &link)?;
            let non_source = path.extension().is_some_and(|ext| {
                ["md", "toml", "json", "yaml", "yml", "just"]
                    .iter()
                    .any(|kind| ext == *kind)
            }) || path.file_name().is_some_and(|name| name == "justfile");
            if non_source {
                continue;
            }
            match source::inspect(&path)? {
                Some(source) => ensure!(
                    source.marker("DECISION", &row.id),
                    "{}: decision marker absent in {}",
                    row.id,
                    path.display()
                ),
                None => console::stderr(format!(
                    "WARNING [{}]: marker inspection unsupported for {}",
                    row.id,
                    path.display()
                )),
            }
        }
    }
    Ok(())
}
fn invariants(context: &Context, memory: &Path) -> Result<()> {
    let rows = table(&memory.join("Invariants.md"), Catalog::Invariants)?;
    details(context, memory, &rows, Catalog::Invariants)?;
    for row in &rows {
        let oracle = format::link(&row.cells[2])?;
        let path = target(&context.root, memory, &oracle.path)?;
        let source = source::inspect(&path)?
            .ok_or_else(|| anyhow::anyhow!("{}: unsupported oracle source language", row.id))?;
        ensure!(
            source.marked_function(&oracles::function_target(context, &row.id)?, &row.id),
            "{}: marked oracle function {} missing or ambiguous",
            row.id,
            oracle.name
        );
        oracles::validate(context, &row.id, &oracle.name, &path)?;
    }
    Ok(())
}
