use anyhow::{Context, Result, ensure};
use signal_hook::{
    consts::{SIGHUP, SIGINT, SIGTERM},
    iterator::Signals,
};
use std::{
    io::IsTerminal,
    os::unix::process::{CommandExt, ExitStatusExt},
    path::Path,
    process::{Child, Command, Output, Stdio},
};

pub fn run(root: &Path, argv: &[String], read_only: bool) -> Result<i32> {
    Ok(exit_code(&execute(root, argv, read_only, false)?))
}
pub fn exit_code(output: &Output) -> i32 {
    output
        .status
        .code()
        .unwrap_or_else(|| 128 + output.status.signal().unwrap_or(1))
}
pub fn execute(root: &Path, argv: &[String], read_only: bool, capture: bool) -> Result<Output> {
    tracked(root, argv, (read_only, capture), None)
}
pub fn tracked(
    root: &Path,
    argv: &[String],
    options: (bool, bool),
    job: Option<&mut discipline_worker::jobs::Job>,
) -> Result<Output> {
    let (read_only, capture) = options;
    ensure!(!argv.is_empty(), "command requires an executable");
    let mut command = if read_only {
        sandbox(root, argv)?
    } else {
        let mut command = Command::new(&argv[0]);
        command.args(&argv[1..]);
        command
    };
    command.current_dir(root);
    if capture {
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
    }
    // Pipes have no foreground-terminal ownership; isolate the process group so
    // a signal sent only to worker also reaches descendants. Interactive children
    // remain in the terminal's foreground group and retain stdin access.
    let group = !std::io::stdin().is_terminal();
    if group {
        command.process_group(0);
    }
    let signals = Signals::new([SIGINT, SIGTERM, SIGHUP])?;
    let child = command
        .spawn()
        .with_context(|| format!("cannot execute {}", argv[0]))?;
    wait(child, signals, group, job)
}
fn wait(
    mut child: Child,
    signals: Signals,
    group: bool,
    job: Option<&mut discipline_worker::jobs::Job>,
) -> Result<Output> {
    let handle = signals.handle();
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
    let forwarding = forward_signals(signals, pid, group);
    let result = child.wait_with_output();
    handle.close();
    forwarding
        .join()
        .map_err(|_| anyhow::anyhow!("signal forwarding thread failed"))?;
    Ok(result?)
}
fn forward_signals(mut signals: Signals, pid: i32, group: bool) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for signal in signals.forever() {
            // The PID belongs to the unreaped child; it cannot be reused while waiting.
            unsafe {
                libc::kill(if group { -pid } else { pid }, signal);
            }
        }
    })
}
fn sandbox(root: &Path, argv: &[String]) -> Result<Command> {
    ensure!(
        cfg!(target_os = "linux"),
        "read-only execution requires the Linux bwrap backend"
    );
    let mut command = Command::new("bwrap");
    command.args([
        "--ro-bind",
        "/",
        "/",
        "--dev",
        "/dev",
        "--proc",
        "/proc",
        "--unshare-all",
        "--die-with-parent",
    ]);
    // Keep projects living in /tmp visible; everywhere remains read-only there.
    let outside_tmp = !root.starts_with("/tmp");
    if outside_tmp {
        command.args(["--tmpfs", "/tmp"]);
    }
    command.arg("--chdir").arg(root).arg("--").args(argv);
    Ok(command)
}
