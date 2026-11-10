use super::architecture::runner;
use super::{config, inventory, selection};
use crate::shared::console;
use crate::shared::paths::patterns::globs;
use crate::structural_lint::rules::{Kind, structure::Structure};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::path::{Component, Path, PathBuf};

// DECISION: D062
#[derive(Clone, Copy)]
enum Exclusion {
    Global,
    Scoped,
}

pub type Inspection = fn(&Path, &Path, &[String]) -> Result<i32>;

// DECISION: D143
pub fn inspection(command: &str) -> Option<Inspection> {
    match command {
        "lint-explain" => Some(run),
        "lint-edges" => Some(edges),
        _ => None,
    }
}

fn run(root: &Path, config_path: &Path, args: &[String]) -> Result<i32> {
    let (path, json) = arguments("lint-explain", args)?;
    let path = path.as_path();
    let config = config::load(root, config_path)?;
    let excluded = crate::shared::paths::patterns::globs(&config.exclude)?;
    let inventory = inventory::collect(root, &excluded)?;
    let globally_excluded = excluded.is_match(path);
    let exclusion = if globally_excluded {
        Exclusion::Global
    } else {
        Exclusion::Scoped
    };
    let mut rows = Vec::new();
    for rule in &config.rules {
        rows.push(explain(rule, path, &inventory, exclusion)?);
    }
    if json {
        console::stdout(json!({"path": path, "rules": rows}));
    } else {
        print(path, &rows);
    }
    Ok(0)
}

// DECISION: D143
fn edges(root: &Path, config_path: &Path, args: &[String]) -> Result<i32> {
    let (path, json) = arguments("lint-edges", args)?;
    let config = config::load(root, config_path)?;
    let inventory = inventory::collect(root, &globs(&config.exclude)?)?;
    let mut rows = Vec::new();
    for rule in &config.rules {
        let architecture = rule.kind == Kind::Structure(Structure::DirectoryArchitecture);
        let measured = match architecture {
            true => runner::edges(root, rule, &inventory, &path)?,
            false => None,
        };
        if let Some(neighbours) = measured {
            rows.push(json!({"id": rule.id, "incoming": neighbours.incoming,
                "outgoing": neighbours.outgoing}));
        }
    }
    ensure!(
        !rows.is_empty(),
        "no enabled directory-architecture rule selects {} as a directory",
        path.display()
    );
    if json {
        console::stdout(json!({"path": path, "rules": rows}));
    } else {
        print_edges(&path, &rows);
    }
    Ok(0)
}

fn print_edges(path: &Path, rows: &[Value]) {
    console::stdout(format!("Measured edges of {}", path.display()));
    for row in rows {
        console::stdout(format!("{}:", row["id"].as_str().unwrap_or("")));
        for side in ["incoming", "outgoing"] {
            let directories = row[side].as_object().cloned().unwrap_or_default();
            console::stdout(format!("  {side}: {} directories", directories.len()));
            for (directory, edges) in directories {
                console::stdout(format!("    {directory}"));
                for edge in edges.as_array().into_iter().flatten() {
                    console::stdout(format!("      {}", edge.as_str().unwrap_or("")));
                }
            }
        }
    }
}

fn arguments(command: &str, args: &[String]) -> Result<(PathBuf, bool)> {
    let (path, flags) = args
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("{command} PATH [--json]"))?;
    ensure!(
        flags.iter().all(|flag| flag == "--json"),
        "unknown {command} argument"
    );
    Ok((selected_path(path)?, !flags.is_empty()))
}

fn selected_path(value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    ensure!(
        !path.is_absolute()
            && !path
                .components()
                .any(|part| matches!(part, Component::ParentDir)),
        "explain path must stay relative to the project"
    );
    let path = path
        .components()
        .filter(|part| !matches!(part, Component::CurDir))
        .collect::<std::path::PathBuf>();
    Ok(match path.as_os_str().is_empty() {
        true => PathBuf::from("."),
        false => path,
    })
}
fn explain(
    rule: &config::Rule,
    path: &Path,
    inventory: &inventory::Inventory,
    exclusion: Exclusion,
) -> Result<Value> {
    let selector = selection::Selector::new(rule)?;
    let mut reason = match (rule.enabled, exclusion) {
        (false, _) => Some("disabled"),
        (_, Exclusion::Global) => Some("global exclude matches"),
        _ => inventory_reason(rule, path, inventory).or_else(|| selector.reason(rule, path)),
    };
    let architecture = rule.kind == super::rules::Kind::Structure(Structure::DirectoryArchitecture);
    let selected_architecture = architecture && reason.is_none();
    if selected_architecture {
        let lacks_sources =
            !super::architecture::runner::contains_directory(rule, path, inventory)?;
        if lacks_sources {
            reason = Some("directory has no selected source files");
        }
    }
    let mut row = json!({"id": rule.id, "kind": rule.kind, "target": rule.target,
        "selected": reason.is_none(), "reason": reason.unwrap_or("selected"),
        "warning_skill": rule.warning_skill, "error_skill": rule.error_skill});
    if architecture {
        row["architecture"] = serde_json::to_value(&rule.architecture)?;
        row["extensions"] = json!(rule.extensions);
    }
    let selected = reason.is_none();
    if selected {
        let effective = selection::effective(rule, path, selector.overrides())?;
        row["warning"] = json!(effective.warning);
        row["error"] = json!(effective.error);
        row["level"] = json!(rule.level.map(|level| level.name()));
        row["matched_overrides"] = json!(effective.overrides);
    }
    Ok(row)
}
fn inventory_reason(
    rule: &config::Rule,
    path: &Path,
    inventory: &inventory::Inventory,
) -> Option<&'static str> {
    let file = inventory.files.iter().any(|entry| entry == path);
    let directory = inventory.directories.contains_key(path);
    let present = file || directory;
    let absent = !present;
    if absent {
        return Some("absent from selected inventory (missing, ignored or non-regular path)");
    }
    let mismatch = (rule.target == "file") != file;
    if mismatch {
        return Some("target type does not match");
    }
    None
}
fn print(path: &Path, rows: &[Value]) {
    console::stdout(format!("Lint selection for {}", path.display()));
    for row in rows {
        console::stdout(format!(
            "{} ({}): {}",
            row["id"].as_str().unwrap_or(""),
            row["kind"].as_str().unwrap_or(""),
            row["reason"].as_str().unwrap_or("")
        ));
        let selected = row["selected"] == true;
        if selected {
            console::stdout(format!(
                "  warning={} error={} level={} matched_overrides={} (zero-based)",
                row["warning"], row["error"], row["level"], row["matched_overrides"]
            ));
        }
    }
}
