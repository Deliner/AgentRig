// DECISION: D219
use super::format::{Catalog, exception_links, table, target};
use crate::project_config::Context;
use crate::shared::fs_with_context as fs;
use anyhow::{Result, ensure};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Entry {
    decision: String,
    rule: String,
    file: PathBuf,
}

pub(super) fn check(context: &Context, memory: &Path, files: &[PathBuf]) -> Result<()> {
    let listed = listed(context, memory)?;
    let carried = carried(context, files)?;
    let root = crate::shared::paths::resolve(&context.root)?;
    let describe = |entry: &Entry| {
        let Entry {
            decision,
            rule,
            file,
        } = entry;
        let file = file.trim_prefix(&root).display();
        format!("{decision} {rule} in {file}")
    };
    let unlisted: Vec<String> = carried.difference(&listed).map(describe).collect();
    ensure!(
        unlisted.is_empty(),
        "an exception counts only when its decision lists it under Applies in as [exception RULE](FILE); list it there or remove the exception: {}",
        unlisted.join(", ")
    );
    let absent: Vec<String> = listed.difference(&carried).map(describe).collect();
    ensure!(
        absent.is_empty(),
        "a decision lists an exception that its file does not carry; remove the entry or restore the exception: {}",
        absent.join(", ")
    );
    Ok(())
}
fn listed(context: &Context, memory: &Path) -> Result<BTreeSet<Entry>> {
    let mut listed = BTreeSet::new();
    for row in table(&memory.join("Decisions.md"), Catalog::Decisions)? {
        for link in exception_links(&row.cells[2]) {
            listed.insert(Entry {
                decision: row.id.clone(),
                rule: link.name,
                file: target(context, memory, &link.path)?,
            });
        }
    }
    Ok(listed)
}
fn carried(context: &Context, files: &[PathBuf]) -> Result<BTreeSet<Entry>> {
    let mut carried = BTreeSet::new();
    for path in files {
        let file = crate::shared::paths::resolve(&context.root.join(path))?;
        let Ok(source) = String::from_utf8(fs::read(&file)?) else {
            continue;
        };
        for exception in crate::shared::source_parsers::exceptions(&file, &source)? {
            carried.insert(Entry {
                decision: exception.decision,
                rule: exception.rule,
                file: file.clone(),
            });
        }
    }
    Ok(carried)
}
