// DECISION: D024
mod apply;
mod external;
pub(super) mod migration;
mod model;
mod operation;
mod plan;
pub(crate) mod recovery;
pub(super) mod release;
mod review;
pub(crate) mod storage;

use anyhow::{Result, bail};
use std::path::Path;

pub fn run(root: &Path, args: &[String]) -> Result<i32> {
    arguments(args)?;
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
        _ => bail!("upgrade plan RELEASE_EXECUTABLE | apply PLAN | rollback"),
    }
}

pub fn arguments(args: &[String]) -> Result<()> {
    let valid = matches!(args, [command, _] if command == "plan" || command == "apply")
        || matches!(args, [command] if command == "rollback");
    anyhow::ensure!(
        valid,
        "upgrade plan RELEASE_EXECUTABLE | apply PLAN | rollback"
    );
    Ok(())
}
