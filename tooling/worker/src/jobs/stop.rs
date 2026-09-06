use super::{owner, scope::Scope, storage};
use anyhow::{Result, ensure};
use std::{
    fs::OpenOptions,
    path::Path,
    time::{Duration, Instant},
};

pub fn run(runtime: &Path, id: &str) -> Result<()> {
    let record = storage::load(runtime, id)?;
    ensure!(
        owner().as_ref() == Some(&record.owner),
        "run belongs to another owner; set WORKER_OWNER to its recorded owner explicitly"
    );
    let scope = record.scope.as_ref().ok_or_else(|| {
        anyhow::anyhow!("run has no scope containment; refusing to claim descendant cleanup")
    })?;
    let populated = scope.observe()?["populated"] == true;
    let already_stopped = !populated;
    if already_stopped {
        return Ok(());
    }
    OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(runtime.join("jobs").join(id).join("stop-requested"))?
        .sync_all()?;
    scope.signal(false)?;
    let stopped = wait(scope)?;
    if stopped {
        return Ok(());
    }
    scope.signal(true)?;
    ensure!(
        wait(scope)?,
        "scope still has live processes after forced termination"
    );
    Ok(())
}
fn wait(scope: &Scope) -> Result<bool> {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        let empty = scope.observe()?["populated"] == false;
        if empty {
            return Ok(true);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(false)
}
