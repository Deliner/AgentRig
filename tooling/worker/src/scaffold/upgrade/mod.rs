// DECISION: D024
mod apply;
mod configuration;
mod external;
pub(super) mod migration;
use super::recovery::model;
mod operation;
mod plan;
pub(crate) use super::recovery;
pub(super) mod release;
mod review;
pub(crate) mod storage;

use anyhow::{Result, bail};
use std::path::Path;

pub fn run(root: &Path, args: &[String]) -> Result<i32> {
    arguments(args)?;
    if let [_, option, path] = args {
        let configuration = option == "--config";
        if configuration {
            return configuration::create(root, Path::new(path));
        }
    }
    if let [command, binary] = args {
        let planning = command == "plan";
        if planning {
            return plan::create(root, Path::new(binary));
        }
    }
    let directory = storage::directory(root)?;
    std::fs::create_dir_all(&directory)?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(directory.join("lock"))?;
    fs2::FileExt::try_lock_exclusive(&lock)?;
    match args {
        [command, path] if command == "apply" => apply::run(root, Path::new(path)),
        [command] if command == "rollback" => apply::rollback(root),
        _ => bail!(
            "upgrade plan RELEASE_EXECUTABLE | plan --config CONFIG_YAML | apply PLAN | rollback"
        ),
    }
}

pub fn arguments(args: &[String]) -> Result<()> {
    let valid = matches!(args, [command, _] if command == "plan" || command == "apply")
        || matches!(args, [command] if command == "rollback")
        || matches!(args, [command, option, _] if command == "plan" && option == "--config");
    anyhow::ensure!(
        valid,
        "upgrade plan RELEASE_EXECUTABLE | plan --config CONFIG_YAML | apply PLAN | rollback"
    );
    Ok(())
}
