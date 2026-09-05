use crate::response::{self, Expected};
use anyhow::{Context, Result};
use serde_json::json;
use std::{
    io::Write,
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

pub struct Broker {
    stopped: Arc<AtomicBool>,
    attempts: Arc<AtomicUsize>,
    exhausted: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<Result<()>>>,
}
impl Broker {
    pub fn start(expected: Expected, work: PathBuf, socket: &Path, limit: usize) -> Result<Self> {
        use std::os::fd::AsRawFd;
        let parent = std::fs::File::open(socket.parent().context("socket directory required")?)?;
        let name = socket
            .file_name()
            .context("socket name required")?
            .to_string_lossy();
        let listener = UnixListener::bind(format!("/proc/self/fd/{}/{name}", parent.as_raw_fd()))?;
        listener.set_nonblocking(true)?;
        let stopped = Arc::new(AtomicBool::new(false));
        let attempts = Arc::new(AtomicUsize::new(0));
        let exhausted = Arc::new(AtomicBool::new(false));
        let counters = (stopped.clone(), attempts.clone(), exhausted.clone());
        let thread = thread::spawn(move || {
            while !counters.0.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let count = counters.1.fetch_add(1, Ordering::SeqCst) + 1;
                        let result = response::file(&work.join("review.json"), &expected);
                        let stop = result.is_err() && count >= limit;
                        counters.2.fetch_or(stop, Ordering::SeqCst);
                        let message = feedback(result.err(), counters.2.load(Ordering::SeqCst));
                        let _ = reply(stream, message);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10))
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            Ok(())
        });
        Ok(Self {
            stopped,
            attempts,
            exhausted,
            thread: Some(thread),
        })
    }
    pub fn finish(mut self) -> Result<(usize, bool)> {
        self.stopped.store(true, Ordering::SeqCst);
        self.thread
            .take()
            .context("validator broker already finished")?
            .join()
            .map_err(|_| anyhow::anyhow!("validator broker panicked"))??;
        Ok((
            self.attempts.load(Ordering::SeqCst),
            self.exhausted.load(Ordering::SeqCst),
        ))
    }
}
impl Drop for Broker {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
fn feedback(error: Option<anyhow::Error>, exhausted: bool) -> serde_json::Value {
    if exhausted {
        return json!({"continue":false,"stopReason":"format attempt limit exhausted"});
    }
    match error {
        Some(error) => {
            json!({"decision":"block","reason":format!("Correct /work/review.json: {error:#}")})
        }
        None => json!({}),
    }
}
fn reply(mut stream: UnixStream, value: serde_json::Value) -> Result<()> {
    stream.set_write_timeout(Some(Duration::from_secs(1)))?;
    stream.write_all(&serde_json::to_vec(&value)?)?;
    Ok(())
}
pub fn hook() -> Result<()> {
    use std::io::Read;
    let mut stream = UnixStream::connect("/review-bin/control.sock")?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    println!("{response}");
    Ok(())
}
