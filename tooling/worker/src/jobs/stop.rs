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
    let pending = scope.invocation.is_none();
    if pending {
        request(runtime, id)?;
        let latest = storage::load(runtime, id)?;
        let adopted = latest
            .scope
            .as_ref()
            .is_some_and(|scope| scope.invocation.is_some());
        if adopted {
            return run(runtime, id);
        }
        return Ok(());
    }
    let populated = scope.observe()?["populated"] == true;
    let already_stopped = !populated;
    if already_stopped {
        return Ok(());
    }
    request(runtime, id)?;
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
fn request(runtime: &Path, id: &str) -> Result<()> {
    OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(runtime.join("jobs").join(id).join("stop-requested"))?
        .sync_all()?;
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
