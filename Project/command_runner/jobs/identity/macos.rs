use super::{Resources, Sample};
use anyhow::{Result, ensure};
use std::{ffi::c_int, mem::MaybeUninit};

pub fn boot() -> Result<String> {
    let mut value = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut size = size_of::<libc::timeval>();
    // DECISION: D066
    let status = unsafe {
        libc::sysctlbyname(
            c"kern.boottime".as_ptr(),
            (&raw mut value).cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    ensure!(
        status == 0,
        "cannot read boot time: {}",
        std::io::Error::last_os_error()
    );
    Ok(format!("{}.{:06}", value.tv_sec, value.tv_usec))
}
pub fn sample(pid: u32) -> Result<Sample> {
    let bsd: libc::proc_bsdinfo = info(pid, libc::PROC_PIDTBSDINFO)?;
    let zombie = bsd.pbi_status == libc::SZOMB;
    let task = info::<libc::proc_taskinfo>(pid, libc::PROC_PIDTASKINFO).ok();
    let cpu = task
        .as_ref()
        .map_or(0, |task| task.pti_total_user + task.pti_total_system);
    Ok(Sample {
        start: bsd.pbi_start_tvsec * 1_000_000 + bsd.pbi_start_tvusec,
        zombie: zombie || task.is_none(),
        resources: Resources {
            cpu_millis: nanoseconds(cpu) / 1_000_000,
            resident_bytes: task.map_or(0, |task| task.pti_resident_size),
        },
    })
}
fn info<T>(pid: u32, flavor: c_int) -> Result<T> {
    // DECISION: D113
    let mut value = MaybeUninit::<T>::zeroed();
    let size = c_int::try_from(size_of::<T>())?;
    // DECISION: D066
    let written = unsafe {
        libc::proc_pidinfo(
            c_int::try_from(pid)?,
            flavor,
            0,
            value.as_mut_ptr().cast(),
            size,
        )
    };
    ensure!(
        written == size,
        "cannot inspect process {pid}: {}",
        std::io::Error::last_os_error()
    );
    // DECISION: D066
    Ok(unsafe { value.assume_init() })
}
// DECISION: D066
#[allow(deprecated)]
fn nanoseconds(ticks: u64) -> u64 {
    let mut timebase = libc::mach_timebase_info { numer: 0, denom: 0 };
    // DECISION: D066
    let known = unsafe { libc::mach_timebase_info(&mut timebase) } == 0 && timebase.denom != 0;
    if known {
        return ticks * u64::from(timebase.numer) / u64::from(timebase.denom);
    }
    ticks
}
