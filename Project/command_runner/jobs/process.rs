use super::{
    Capture, Grouping, Job,
    streams::{Readers, Sink},
};
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
pub fn execute(command: &mut Command, capture: Capture, job: Option<&mut Job>) -> Result<Output> {
    if let Some(job) = job.as_ref() {
        command
            .env("WORKER_OWNER", &job.record().owner)
            .env("WORKER_PARENT_RUN", &job.record().run_id);
    }
    let logs = job.as_ref().map(|job| job.open_logs()).transpose()?;
    let (grouping, sink) = arrange(command, capture, job.as_deref());
    let signals = Signals::new([SIGINT, SIGTERM, SIGHUP])?;
    let handle = signals.handle();
    let mut child = command
        .spawn()
        .with_context(|| format!("cannot execute {}", command.get_program().to_string_lossy()))?;
    let pid = child.id() as i32;
    attach(&mut child, job, grouping)?;
    let forwarding = forward_signals(signals, pid, grouping);
    let readers = Readers::start(&mut child, logs, sink);
    let status = child.wait();
    handle.close();
    forwarding
        .join()
        .map_err(|_| anyhow::anyhow!("signal forwarding thread failed"))?;
    readers.finish(status)
}
fn arrange(command: &mut Command, capture: Capture, job: Option<&Job>) -> (Grouping, Sink) {
    let logged = job.is_some();
    let forward = job.is_none_or(|job| !job.record().background);
    let collecting = capture.collects();
    let piped = collecting || logged;
    if collecting {
        command.stdin(Stdio::null());
    }
    if piped {
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
    }
    let leads_group = forward && !std::io::stdin().is_terminal();
    let grouping = if leads_group {
        Grouping::Own
    } else {
        Grouping::Inherited
    };
    if leads_group {
        command.process_group(0);
    }
    let sink = if collecting {
        Sink::Collect
    } else if forward {
        Sink::Mirror
    } else {
        Sink::LogOnly
    };
    (grouping, sink)
}

fn forward_signals(
    mut signals: Signals,
    pid: i32,
    grouping: Grouping,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for signal in signals.forever() {
            unsafe {
                libc::kill(grouping.target(pid), signal);
            }
        }
    })
}

fn attach(
    child: &mut std::process::Child,
    job: Option<&mut Job>,
    grouping: Grouping,
) -> Result<()> {
    let pid = child.id() as i32;
    let attached = job.map(|job| job.attach(child.id(), grouping)).transpose();
    if let Err(error) = attached {
        unsafe {
            libc::kill(grouping.target(pid), SIGTERM);
        }
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    Ok(())
}
