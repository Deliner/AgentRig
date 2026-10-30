use super::{Resources, Sample};
use anyhow::{Context, Result};
use std::fs;

pub fn boot() -> Result<String> {
    Ok(fs::read_to_string("/proc/sys/kernel/random/boot_id")?
        .trim()
        .into())
}
pub fn sample(pid: u32) -> Result<Sample> {
    let source = fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let (_, fields) = source.rsplit_once(") ").context("invalid process stat")?;
    let fields: Vec<_> = fields.split_whitespace().collect();
    let number = |index: usize| -> Result<u64> {
        Ok(fields
            .get(index)
            .context("incomplete process stat")?
            .parse()?)
    };
    // DECISION: D066
    let page_size = u64::try_from(unsafe { libc::sysconf(libc::_SC_PAGESIZE) })?;
    // DECISION: D066
    let ticks = u64::try_from(unsafe { libc::sysconf(libc::_SC_CLK_TCK) })?;
    Ok(Sample {
        start: number(19)?,
        zombie: matches!(fields.first(), Some(&"Z") | Some(&"X")),
        resources: Resources {
            cpu_millis: (number(11)? + number(12)?) * 1000 / ticks.max(1),
            resident_bytes: number(21)? * page_size,
        },
    })
}
