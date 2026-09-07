use super::{config, inventory, selection};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::path::{Component, Path, PathBuf};

pub fn run(root: &Path, config_path: &Path, args: &[String]) -> Result<i32> {
    let mut args = args.to_vec();
    let selected = super::cli::selection(root, &mut args)?;
    let (path, flags) = args
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("lint-explain PATH [--json]"))?;
    ensure!(
        flags.iter().all(|flag| flag == "--json"),
        "unknown lint-explain argument"
    );
    let path = selected_path(path)?;
    let path = path.as_path();
    let config = config::load(root, config_path)?;
    let excluded = config::globs(&config.exclude)?;
    let source = selected
        .as_ref()
        .map(|(_, backend)| backend.repository_source(root))
        .transpose()?
        .flatten();
    let inventory = inventory::collect_source(root, &excluded, source.as_ref())?;
    let global = excluded.is_match(path);
    let mut rows = Vec::new();
    for rule in &config.rules {
        rows.push(explain(rule, path, &inventory, global)?);
    }
    let json = !flags.is_empty();
    if json {
        println!("{}", json!({"path": path, "rules": rows}));
    } else {
        print(path, &rows);
    }
    Ok(0)
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
    global: bool,
) -> Result<Value> {
    let selector = selection::Selector::new(rule)?;
    let mut reason = match (rule.enabled, global) {
        (false, _) => Some("disabled"),
        (_, true) => Some("global exclude matches"),
        _ => inventory_reason(rule, path, inventory).or_else(|| selector.reason(rule, path)),
    };
    let architecture = rule.kind == super::rules::Kind::DirectoryArchitecture;
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
        let effective = selection::effective(rule, path, &selector.overrides)?;
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
    println!("Lint selection for {}", path.display());
    for row in rows {
        println!(
            "{} ({}): {}",
            row["id"].as_str().unwrap_or(""),
            row["kind"].as_str().unwrap_or(""),
            row["reason"].as_str().unwrap_or("")
        );
        let selected = row["selected"] == true;
        if selected {
            println!(
                "  warning={} error={} level={} matched_overrides={} (zero-based)",
                row["warning"], row["error"], row["level"], row["matched_overrides"]
            );
        }
    }
}
