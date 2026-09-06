use super::{Job, streams::Readers};
use anyhow::{Context, Result};
use signal_hook::{
    consts::{SIGHUP, SIGINT, SIGTERM},
    iterator::Signals,
};
use std::{
    io::IsTerminal,
    os::unix::process::{CommandExt, ExitStatusExt},
    process::{Command, Output, Stdio},
};

pub fn exit_code(output: &Output) -> i32 {
    output
        .status
        .code()
        .unwrap_or_else(|| 128 + output.status.signal().unwrap_or(1))
}
pub fn execute(command: &mut Command, capture: bool, job: Option<&mut Job>) -> Result<Output> {
    let logged = job.is_some();
    let forward = job.as_ref().is_none_or(|job| job.record().scope.is_none());
    let piped = capture || logged;
    if capture {
        command.stdin(Stdio::null());
    }
    if piped {
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
    }
    let group = !std::io::stdin().is_terminal();
    if group {
        command.process_group(0);
    }
    let logs = job.as_ref().map(|job| job.open_logs()).transpose()?;
    let signals = Signals::new([SIGINT, SIGTERM, SIGHUP])?;
    let handle = signals.handle();
    let mut child = command
        .spawn()
        .with_context(|| format!("cannot execute {}", command.get_program().to_string_lossy()))?;
    let pid = child.id() as i32;
    attach(&mut child, job, group)?;
    let forwarding = forward_signals(signals, pid, group);
    let readers = Readers::start(&mut child, capture, logs, forward);
    let status = child.wait();
    handle.close();
    forwarding
        .join()
        .map_err(|_| anyhow::anyhow!("signal forwarding thread failed"))?;
    readers.finish(status)
}

fn forward_signals(mut signals: Signals, pid: i32, group: bool) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for signal in signals.forever() {
            // The PID belongs to the unreaped child while forwarding is active.
            unsafe {
                libc::kill(if group { -pid } else { pid }, signal);
            }
        }
    })
}

fn attach(child: &mut std::process::Child, job: Option<&mut Job>, group: bool) -> Result<()> {
    let pid = child.id() as i32;
    let attached = job.map(|job| job.attach(child.id(), group)).transpose();
    if let Err(error) = attached {
        unsafe {
            libc::kill(if group { -pid } else { pid }, SIGTERM);
        }
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    Ok(())
}
