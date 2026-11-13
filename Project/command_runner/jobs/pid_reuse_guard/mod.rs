// DECISION: D034
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Identity {
    pub pid: u32,
    pub start_ticks: u64,
    pub boot: String,
}
#[derive(Serialize)]
pub struct Resources {
    pub cpu_millis: u64,
    pub resident_bytes: u64,
}
struct Sample {
    start: u64,
    zombie: bool,
    resources: Resources,
}
impl Identity {
    pub fn read(pid: u32) -> Result<Self> {
        Ok(Self {
            pid,
            start_ticks: sample(pid)?.start,
            boot: boot()?,
        })
    }
    pub fn observe(&self) -> Option<Resources> {
        let boot = boot().ok()?;
        let sample = sample(self.pid).ok()?;
        let matches = boot == self.boot && sample.start == self.start_ticks && !sample.zombie;
        matches.then_some(sample.resources)
    }
    pub fn replaced(&self) -> bool {
        let boot = boot().ok();
        let sample = sample(self.pid).ok();
        let rebooted = boot.is_some_and(|boot| boot != self.boot);
        rebooted || sample.is_some_and(|sample| sample.start != self.start_ticks)
    }
}

fn boot() -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        linux::boot()
    }
    #[cfg(target_os = "macos")]
    {
        macos::boot()
    }
}
fn sample(pid: u32) -> Result<Sample> {
    #[cfg(target_os = "linux")]
    {
        linux::sample(pid)
    }
    #[cfg(target_os = "macos")]
    {
        macos::sample(pid)
    }
}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
