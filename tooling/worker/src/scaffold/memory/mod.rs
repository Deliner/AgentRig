// DECISION: D019
// DECISION: D016
// DECISION: D013
// DECISION: D012
// DECISION: D009
// DECISION: D008
// DECISION: D002
pub(super) mod format;
mod history;
mod oracles;
mod resume;
mod source;
pub use resume::run as resume;

use super::config::Context;
use anyhow::{Result, ensure};
use format::{Row, sections, table, target};
use std::{
    collections::{HashMap, HashSet},
    fs,
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
    invariants(context, &memory)?;
    sections(&memory.join("State.md"), format::STATE_SECTIONS, None)?;
    Ok(0)
}
fn details(context: &Context, memory: &Path, rows: &[Row], directory: &str) -> Result<()> {
    let (headings, optional) = format::detail_sections(directory);
    let mut known = HashSet::new();
    for row in rows {
        ensure!(
            row.detail == format!("{directory}/{}.md", &row.id[1..]),
            "{}: detail path does not match ID",
            row.id
        );
        known.insert(row.detail.clone());
        sections(
            &target(&context.root, memory, &row.detail)?,
            headings,
            optional,
        )?;
    }
    reject_unindexed(memory, directory, &known)
}
fn reject_unindexed(memory: &Path, directory: &str, known: &HashSet<String>) -> Result<()> {
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
                ensure!(
                    known.contains(&format!(
                        "{directory}/{}",
                        path.file_name().unwrap().to_string_lossy()
                    )),
                    "unindexed detail {}",
                    path.display()
                );
            }
        }
    }
    Ok(())
}
fn backlog(context: &Context, memory: &Path) -> Result<()> {
    let path = memory.join("Backlog.md");
    // Existing consumers may not have adopted Backlog yet.
    let present = path.try_exists()?;
    if present {
        details(context, memory, &table(&path, 'B')?, "Backlog")
    } else {
        reject_unindexed(memory, "Backlog", &HashSet::new())
    }
}
fn plan(context: &Context, memory: &Path) -> Result<()> {
    let rows = table(&memory.join("Plan.md"), 'P')?;
    details(context, memory, &rows, "Plan")?;
    ensure!(
        rows.iter().filter(|row| row.cells[1] == "active").count() <= 1,
        "Plan has multiple active features"
    );
    let index: HashMap<_, _> = rows.iter().map(|row| (row.id.as_str(), row)).collect();
    let mut graph = HashMap::new();
    for row in &rows {
        plan_status(row, memory)?;
        graph.insert(row.id.as_str(), dependencies(row, &index)?);
    }
    acyclic(&graph)
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
fn acyclic(graph: &HashMap<&str, Vec<&str>>) -> Result<()> {
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
    let rows = table(&memory.join("Decisions.md"), 'D')?;
    details(context, memory, &rows, "Decisions")?;
    for row in &rows {
        let links = format::links(&row.cells[2]);
        ensure!(!links.is_empty(), "{}: application link required", row.id);
        for link in links {
            let path = target(&context.root, memory, &link)?;
            let non_source = path.extension().is_some_and(|ext| {
                ["md", "toml", "json", "yaml", "yml"]
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
                None => eprintln!(
                    "WARNING [{}]: marker inspection unsupported for {}",
                    row.id,
                    path.display()
                ),
            }
        }
    }
    Ok(())
}
fn invariants(context: &Context, memory: &Path) -> Result<()> {
    let rows = table(&memory.join("Invariants.md"), 'I')?;
    details(context, memory, &rows, "Invariants")?;
    for row in &rows {
        let (name, link) = format::link(&row.cells[2])?;
        let path = target(&context.root, memory, &link)?;
        let source = source::inspect(&path)?
            .ok_or_else(|| anyhow::anyhow!("{}: unsupported oracle source language", row.id))?;
        ensure!(
            source.marked_function(&oracles::function_target(context, &row.id)?, &row.id),
            "{}: marked oracle function {name} missing or ambiguous",
            row.id
        );
        oracles::validate(context, &row.id, &name, &path)?;
    }
    Ok(())
}
