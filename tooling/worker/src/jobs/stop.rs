use super::{group, identity::Identity, owner, storage};
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
    ensure!(
        record.background,
        "run has no process group containment; refusing to claim descendant cleanup"
    );
    let Some(leader) = &record.group else {
        request(runtime, id)?;
        let adopted = storage::load(runtime, id)?.group.is_some();
        if adopted {
            return run(runtime, id);
        }
        return Ok(());
    };
    let already_stopped = !group::populated(leader);
    if already_stopped {
        return Ok(());
    }
    request(runtime, id)?;
    group::signal(leader, false)?;
    let stopped = wait(leader);
    if stopped {
        return Ok(());
    }
    group::signal(leader, true)?;
    ensure!(
        wait(leader),
        "process group still has live processes after forced termination"
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
fn wait(leader: &Identity) -> bool {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        let empty = !group::populated(leader);
        if empty {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}
