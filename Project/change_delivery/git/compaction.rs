// DECISION: D230
use crate::project_config::Context;
use crate::shared::atomic_files::{load_json, save_json};
use anyhow::Result;
use std::path::{Path, PathBuf};

const RECORD: &str = "compaction";
const INSTRUCTION: &str = include_str!("compaction.txt");

fn location(context: &Context) -> Result<PathBuf> {
    Ok(context.path(&context.config.paths.runtime)?.join(RECORD))
}

fn required(location: &Path) -> Result<bool> {
    Ok(load_json::<bool>(location)?.is_some_and(|required| required))
}

pub(super) fn instruction() -> &'static str {
    INSTRUCTION.trim_end()
}

pub(super) fn require(context: &Context) -> Result<()> {
    save_json(&location(context)?, &true)
}

pub fn due(context: &Context) -> Result<Option<&'static str>> {
    Ok(required(&location(context)?)?.then(instruction))
}

pub fn lift(context: &Context) -> Result<()> {
    let location = location(context)?;
    let pending = required(&location)?;
    if pending {
        save_json(&location, &false)?;
    }
    Ok(())
}
