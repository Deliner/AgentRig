use crate::shared::fs::File;
use anyhow::Result;
use std::{
    any::Any,
    io::{Read, Write},
    os::unix::io::{AsRawFd, RawFd},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

// DECISION: D062
#[derive(Clone, Copy)]
pub enum Sink {
    Collect,
    Mirror,
    LogOnly,
}
#[derive(Clone, Copy)]
pub(super) enum Channel {
    Stdout,
    Stderr,
}
pub struct Stream {
    pub log: Option<File>,
    pub sink: Sink,
    pub channel: Channel,
}
pub struct Logs {
    pub stdout: File,
    pub stderr: File,
}
pub struct Readers {
    stdout: Option<JoinHandle<Result<Vec<u8>>>>,
    stderr: Option<JoinHandle<Result<Vec<u8>>>>,
    exited: Arc<AtomicBool>,
}
impl Readers {
    pub fn start(child: &mut std::process::Child, logs: Option<Logs>, sink: Sink) -> Self {
        let (stdout_log, stderr_log) = match logs {
            Some(logs) => (Some(logs.stdout), Some(logs.stderr)),
            None => (None, None),
        };
        let exited = Arc::new(AtomicBool::new(false));
        Self {
            stdout: child.stdout.take().map(|pipe| {
                drain(
                    pipe,
                    Stream {
                        log: stdout_log,
                        sink,
                        channel: Channel::Stdout,
                    },
                    Arc::clone(&exited),
                )
            }),
            stderr: child.stderr.take().map(|pipe| {
                drain(
                    pipe,
                    Stream {
                        log: stderr_log,
                        sink,
                        channel: Channel::Stderr,
                    },
                    Arc::clone(&exited),
                )
            }),
            exited,
        }
    }
    pub fn mark_exited(&self) {
        // DECISION: D108
        self.exited.store(true, Ordering::Release);
    }
    pub fn finish(
        self,
        status: std::io::Result<std::process::ExitStatus>,
    ) -> Result<std::process::Output> {
        let stdout = join(self.stdout);
        let stderr = join(self.stderr);
        Ok(std::process::Output {
            status: status?,
            stdout: stdout?,
            stderr: stderr?,
        })
    }
}
pub fn drain(
    input: impl Read + AsRawFd + Send + 'static,
    stream: Stream,
    exited: Arc<AtomicBool>,
) -> JoinHandle<Result<Vec<u8>>> {
    thread::spawn(move || copy(input, stream, exited))
}
fn copy(
    mut input: impl Read + AsRawFd,
    mut stream: Stream,
    exited: Arc<AtomicBool>,
) -> Result<Vec<u8>> {
    let mut captured = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        let ready = wait_readable(input.as_raw_fd(), &exited)?;
        let done = !ready;
        if done {
            break;
        }
        let count = input.read(&mut buffer)?;
        let eof = count == 0;
        if eof {
            break;
        }
        let bytes = &buffer[..count];
        if let Some(log) = &mut stream.log {
            log.write_all(bytes)?;
        }
        match stream.sink {
            Sink::Collect => captured.extend_from_slice(bytes),
            Sink::Mirror => forward(bytes, stream.channel)?,
            Sink::LogOnly => {}
        }
    }
    if let Some(log) = stream.log {
        log.sync_all()?;
    }
    Ok(captured)
}
fn wait_readable(fd: RawFd, exited: &AtomicBool) -> Result<bool> {
    loop {
        // DECISION: D108
        let finished = exited.load(Ordering::Acquire);
        let mut descriptor = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        // DECISION: D095
        let count = unsafe { libc::poll(&mut descriptor, 1, 200) };
        let failed = count < 0;
        if failed {
            let error = std::io::Error::last_os_error();
            // DECISION: D156
            let interrupted = error.kind() == std::io::ErrorKind::Interrupted;
            if interrupted {
                continue;
            }
            return Err(error.into());
        }
        let ready = descriptor.revents != 0;
        if ready {
            return Ok(true);
        }
        if finished {
            return Ok(false);
        }
    }
}
fn forward(bytes: &[u8], channel: Channel) -> Result<()> {
    match channel {
        Channel::Stderr => std::io::stderr().write_all(bytes)?,
        Channel::Stdout => std::io::stdout().write_all(bytes)?,
    }
    Ok(())
}
pub fn join(thread: Option<JoinHandle<Result<Vec<u8>>>>) -> Result<Vec<u8>> {
    match thread {
        Some(thread) => thread.join().map_err(|payload| {
            anyhow::anyhow!("output reader panicked: {}", panic_text(payload.as_ref()))
        })?,
        None => Ok(Vec::new()),
    }
}
// DECISION: D129
pub fn panic_text(payload: &(dyn Any + Send)) -> &str {
    match payload.downcast_ref::<String>() {
        Some(text) => text,
        None => payload
            .downcast_ref::<&str>()
            .copied()
            .unwrap_or("panic without a text payload"),
    }
}
