use anyhow::Result;
use std::{
    fs::File,
    io::{Read, Write},
    thread::{self, JoinHandle},
};

pub struct Stream {
    pub log: Option<File>,
    pub capture: bool,
    pub stderr: bool,
}
pub struct Readers {
    stdout: Option<JoinHandle<Result<Vec<u8>>>>,
    stderr: Option<JoinHandle<Result<Vec<u8>>>>,
}
impl Readers {
    pub fn start(
        child: &mut std::process::Child,
        capture: bool,
        logs: Option<(File, File)>,
    ) -> Self {
        let (stdout_log, stderr_log) = match logs {
            Some((out, err)) => (Some(out), Some(err)),
            None => (None, None),
        };
        Self {
            stdout: child.stdout.take().map(|pipe| {
                drain(
                    pipe,
                    Stream {
                        log: stdout_log,
                        capture,
                        stderr: false,
                    },
                )
            }),
            stderr: child.stderr.take().map(|pipe| {
                drain(
                    pipe,
                    Stream {
                        log: stderr_log,
                        capture,
                        stderr: true,
                    },
                )
            }),
        }
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
pub fn drain(input: impl Read + Send + 'static, stream: Stream) -> JoinHandle<Result<Vec<u8>>> {
    thread::spawn(move || copy(input, stream))
}
fn copy(mut input: impl Read, mut stream: Stream) -> Result<Vec<u8>> {
    let mut captured = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        let count = input.read(&mut buffer)?;
        let eof = count == 0;
        if eof {
            break;
        }
        let bytes = &buffer[..count];
        if let Some(log) = &mut stream.log {
            log.write_all(bytes)?;
        }
        if stream.capture {
            captured.extend_from_slice(bytes);
        } else if stream.stderr {
            std::io::stderr().write_all(bytes)?;
        } else {
            std::io::stdout().write_all(bytes)?;
        }
    }
    if let Some(log) = stream.log {
        log.sync_all()?;
    }
    Ok(captured)
}
pub fn join(thread: Option<JoinHandle<Result<Vec<u8>>>>) -> Result<Vec<u8>> {
    match thread {
        Some(thread) => thread
            .join()
            .map_err(|_| anyhow::anyhow!("output reader failed"))?,
        None => Ok(Vec::new()),
    }
}
