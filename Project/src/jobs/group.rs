// DECISION: D034
use super::identity::Identity;
use anyhow::{Result, ensure};
use serde_json::{Value, json};

pub fn populated(leader: &Identity) -> bool {
    let group = -i32::try_from(leader.pid).unwrap_or(i32::MAX);
    let present = unsafe { libc::kill(group, 0) } == 0
        || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM);
    present && !leader.replaced()
}
pub fn observe(leader: &Identity) -> Value {
    json!({"leader": leader.pid, "populated": populated(leader),
        "leader_resources": leader.observe()})
}
pub fn signal(leader: &Identity, force: bool) -> Result<()> {
    ensure!(
        populated(leader),
        "process group identity changed; refusing to stop a replacement"
    );
    let signal = if force { libc::SIGKILL } else { libc::SIGTERM };
    let sent = unsafe { libc::killpg(i32::try_from(leader.pid)?, signal) } == 0;
    ensure!(
        sent,
        "process group stop failed: {}",
        std::io::Error::last_os_error()
    );
    Ok(())
}
pub fn current(leader: &Identity) -> bool {
    let group = unsafe { libc::getpgrp() };
    i32::try_from(leader.pid).is_ok_and(|pid| pid == group)
}
