// DECISION: D034
use super::identity::Identity;
use anyhow::{Result, ensure};
use serde_json::{Value, json};

// DECISION: D062
#[derive(Clone, Copy)]
pub enum Stop {
    Graceful,
    Forceful,
}

pub fn populated(leader: &Identity) -> bool {
    let group = -i32::try_from(leader.pid).unwrap_or(i32::MAX);
    // DECISION: D066
    let present = unsafe { libc::kill(group, 0) } == 0
        || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM);
    present && !leader.replaced()
}
pub fn observe(leader: &Identity) -> Value {
    json!({"leader": leader.pid, "populated": populated(leader),
        "leader_resources": leader.observe()})
}
pub fn signal(leader: &Identity, stop: Stop) -> Result<()> {
    ensure!(
        populated(leader),
        "process group identity changed; refusing to stop a replacement"
    );
    let signal = match stop {
        Stop::Forceful => libc::SIGKILL,
        Stop::Graceful => libc::SIGTERM,
    };
    // DECISION: D066
    let sent = unsafe { libc::killpg(i32::try_from(leader.pid)?, signal) } == 0;
    ensure!(
        sent,
        "process group stop failed: {}",
        std::io::Error::last_os_error()
    );
    Ok(())
}
pub fn current(leader: &Identity) -> bool {
    // DECISION: D066
    let group = unsafe { libc::getpgrp() };
    i32::try_from(leader.pid).is_ok_and(|pid| pid == group)
}
