// DECISION: D034
use super::jobs::{Capture, Job};
use crate::project_config::Access;
use anyhow::{Context, Result, ensure};
use std::{
    io::ErrorKind,
    path::Path,
    process::{Command, Output},
};

#[derive(Clone, Copy)]
pub struct Sandboxing<'a> {
    pub access: Access,
    pub capture: Capture,
    pub cargo_target: Option<&'a Path>,
    pub rustflags: &'a [String],
}
// DECISION: D182
pub fn compile_with(command: &mut Command, rustflags: &[String]) {
    command.env("CARGO_ENCODED_RUSTFLAGS", rustflags.join("\u{1f}"));
}
pub fn tracked(
    root: &Path,
    argv: &[String],
    options: Sandboxing<'_>,
    job: Option<&mut Job>,
) -> Result<Output> {
    ensure!(!argv.is_empty(), "command requires an executable");
    let scratch = tempfile::tempdir()?;
    let command = match options.access {
        Access::ReadOnly => sandbox(root, argv, &scratch)?,
        Access::Writable => direct(argv)?,
    };
    launch(command, root, options, job)
}
// DECISION: D193
pub fn confined(root: &Path, argv: &[String], options: Sandboxing<'_>) -> Result<Option<Output>> {
    let read_only = match options.access {
        Access::ReadOnly => true,
        Access::Writable => false,
    };
    let refused = read_only && refuses_writes(root)?;
    if refused {
        return launch(direct(argv)?, root, options, None).map(Some);
    }
    Ok(None)
}
fn refuses_writes(directory: &Path) -> Result<bool> {
    let Err(error) = tempfile::tempfile_in(directory) else {
        return Ok(false);
    };
    // DECISION: D193
    let refused = matches!(
        error.kind(),
        ErrorKind::PermissionDenied | ErrorKind::ReadOnlyFilesystem
    );
    if refused { Ok(true) } else { Err(error.into()) }
}
fn direct(argv: &[String]) -> Result<Command> {
    let (program, arguments) = argv
        .split_first()
        .context("command requires an executable")?;
    let mut command = Command::new(program);
    command.args(arguments);
    Ok(command)
}
fn launch(
    mut command: Command,
    root: &Path,
    options: Sandboxing<'_>,
    job: Option<&mut Job>,
) -> Result<Output> {
    command.current_dir(root);
    compile_with(&mut command, options.rustflags);
    // DECISION: D160
    if let Some(target) = options.cargo_target {
        command.env("CARGO_TARGET_DIR", target);
    }
    crate::command_runner::jobs::process::execute(&mut command, options.capture, job)
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
    let scratch = crate::shared::fs_with_context::canonicalize(scratch.path())?;
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
