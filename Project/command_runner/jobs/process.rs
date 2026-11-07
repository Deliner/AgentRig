use super::{
    Capture, Grouping, Job,
    streams::{self, Readers, Sink},
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
    let Arrangement { grouping, sink } = arrange(command, capture, job.as_deref());
    let signals = Signals::new([SIGINT, SIGTERM, SIGHUP])?;
    let handle = signals.handle();
    let mut child = command
        .spawn()
        .with_context(|| format!("cannot execute {}", command.get_program().to_string_lossy()))?;
    let pid = i32::try_from(child.id())?;
    attach(&mut child, job, grouping)?;
    let forwarding = forward_signals(signals, pid, grouping);
    let readers = Readers::start(&mut child, logs, sink);
    let status = child.wait();
    readers.mark_exited();
    handle.close();
    forwarding.join().map_err(|payload| {
        anyhow::anyhow!(
            "signal forwarding thread panicked: {}",
            streams::panic_text(payload.as_ref())
        )
    })?;
    restore_default_signals();
    readers.finish(status)
}
// DECISION: D095
fn restore_default_signals() {
    for signal in [SIGINT, SIGTERM, SIGHUP] {
        // DECISION: D113
        unsafe {
            libc::signal(signal, libc::SIG_DFL);
        }
    }
}
struct Arrangement {
    grouping: Grouping,
    sink: Sink,
}
fn arrange(command: &mut Command, capture: Capture, job: Option<&Job>) -> Arrangement {
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
        command.process_group(0);
        Grouping::Own
    } else {
        Grouping::Inherited
    };
    let sink = if collecting {
        Sink::Collect
    } else if forward {
        Sink::Mirror
    } else {
        Sink::LogOnly
    };
    Arrangement { grouping, sink }
}

fn forward_signals(
    mut signals: Signals,
    pid: i32,
    grouping: Grouping,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for signal in signals.forever() {
            // DECISION: D113
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
    let pid = i32::try_from(child.id())?;
    let attached = job.map(|job| job.attach(child.id(), grouping)).transpose();
    if let Err(error) = attached {
        // DECISION: D113
        unsafe {
            libc::kill(grouping.target(pid), SIGTERM);
        }
        // DECISION: D091
        child.kill().ok();
        // DECISION: D091
        child.wait().ok();
        return Err(error);
    }
    Ok(())
}
