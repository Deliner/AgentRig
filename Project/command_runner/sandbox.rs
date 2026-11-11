// DECISION: D034
use super::jobs::Capture;
use crate::project_config::Access;
use anyhow::{Result, ensure};
use std::{
    path::Path,
    process::{Command, Output},
};

pub struct Sandboxing<'a> {
    pub access: Access,
    pub capture: Capture,
    pub cargo_target: Option<&'a Path>,
}
pub fn tracked(
    root: &Path,
    argv: &[String],
    options: Sandboxing<'_>,
    job: Option<&mut crate::command_runner::jobs::Job>,
) -> Result<Output> {
    let Sandboxing {
        access,
        capture,
        cargo_target,
    } = options;
    ensure!(!argv.is_empty(), "command requires an executable");
    let scratch = tempfile::tempdir()?;
    let mut command = match access {
        Access::ReadOnly => sandbox(root, argv, &scratch)?,
        Access::Writable => {
            let mut command = Command::new(&argv[0]);
            command.args(&argv[1..]);
            command
        }
    };
    command.current_dir(root);
    // DECISION: D160
    if let Some(target) = cargo_target {
        command.env("CARGO_TARGET_DIR", target);
    }
    crate::command_runner::jobs::process::execute(&mut command, capture, job)
}
pub fn available() -> bool {
    let probe = tempfile::tempdir().and_then(|scratch| {
        sandbox(Path::new("/"), &["true".into()], &scratch)
            .map_err(std::io::Error::other)?
            .output()
    });
    probe.is_ok_and(|output| output.status.success())
}

#[cfg(target_os = "linux")]
fn sandbox(root: &Path, argv: &[String], _scratch: &tempfile::TempDir) -> Result<Command> {
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
    let outside_tmp = !root.starts_with("/tmp");
    if outside_tmp {
        command.args(["--tmpfs", "/tmp"]);
    }
    command.arg("--chdir").arg(root).arg("--").args(argv);
    Ok(command)
}

#[cfg(target_os = "macos")]
fn sandbox(_root: &Path, argv: &[String], scratch: &tempfile::TempDir) -> Result<Command> {
    let scratch = crate::shared::fs::canonicalize(scratch.path())?;
    let profile = format!(
        "(version 1)(allow default)(deny network*)(deny file-write*)\
         (allow file-write* (literal \"/dev/null\") (literal \"/dev/zero\") \
         (literal \"/dev/dtracehelper\") (regex #\"^/dev/tty\") (regex #\"^/dev/fd/\") \
         (subpath {}))",
        quoted(&scratch.to_string_lossy())
    );
    let mut command = Command::new("/usr/bin/sandbox-exec");
    command
        .arg("-p")
        .arg(profile)
        .arg("--")
        .args(argv)
        .env("TMPDIR", &scratch);
    Ok(command)
}

#[cfg(target_os = "macos")]
fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
