use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Clone, Deserialize, Serialize)]
pub struct Identity {
    pub pid: u32,
    pub start_ticks: u64,
    pub boot: String,
}
#[derive(Serialize)]
pub struct Resources {
    pub cpu_ticks: u64,
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
            boot: fs::read_to_string("/proc/sys/kernel/random/boot_id")?
                .trim()
                .into(),
        })
    }
    pub fn observe(&self) -> Option<Resources> {
        let boot = fs::read_to_string("/proc/sys/kernel/random/boot_id").ok()?;
        let sample = sample(self.pid).ok()?;
        let matches =
            boot.trim() == self.boot && sample.start == self.start_ticks && !sample.zombie;
        matches.then_some(sample.resources)
    }
}
fn sample(pid: u32) -> Result<Sample> {
    let source = fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let (_, fields) = source.rsplit_once(") ").context("invalid process stat")?;
    let fields: Vec<_> = fields.split_whitespace().collect();
    let number = |index: usize| -> Result<u64> {
        Ok(fields
            .get(index)
            .context("incomplete process stat")?
            .parse()?)
    };
    // /proc stat field 3 starts at index zero; comm may itself contain spaces.
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    Ok(Sample {
        start: number(19)?,
        zombie: matches!(fields.first(), Some(&"Z") | Some(&"X")),
        resources: Resources {
            cpu_ticks: number(11)? + number(12)?,
            resident_bytes: number(21)? * u64::try_from(page_size)?,
        },
    })
}
