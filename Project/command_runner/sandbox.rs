// DECISION: D034
use super::jobs::{Capture, Job};
use crate::project_config::Access;
use crate::shared::fs_with_context::canonicalize as canonical;
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
// DECISION: D204
pub const CATALOG_ROOT: &str = "WORKER_CATALOG_ROOT";
// DECISION: D204
pub fn catalog_run(root: &Path) -> Result<bool> {
    let Some(marked) = std::env::var_os(CATALOG_ROOT) else {
        return Ok(false);
    };
    Ok(Path::new(&marked) == canonical(root)?)
}
// DECISION: D203
enum Confinement {
    Reading,
    Writing,
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
        Access::ReadOnly => sandbox(root, argv, scratch.path(), &Confinement::Reading)?,
        Access::Project => sandbox(root, argv, scratch.path(), &Confinement::Writing)?,
        Access::Unconfined => direct(argv)?,
    };
    launch(command, root, options, job)
}
// DECISION: D193
pub fn confined(root: &Path, argv: &[String], options: Sandboxing<'_>) -> Result<Option<Output>> {
    let refused = options.access.read_only() && refuses_writes(root)?;
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
    // DECISION: D204
    command.env(CATALOG_ROOT, canonical(root)?);
    compile_with(&mut command, options.rustflags);
    // DECISION: D160
    if let Some(target) = options.cargo_target {
        command.env("CARGO_TARGET_DIR", target);
    }
    crate::command_runner::jobs::process::execute(&mut command, options.capture, job)
}
pub fn available() -> bool {
    let probe = tempfile::tempdir().and_then(|scratch| {
        let reading = &Confinement::Reading;
        sandbox(Path::new("/"), &["true".into()], scratch.path(), reading)
            .map_err(std::io::Error::other)?
            .output()
    });
    probe.is_ok_and(|output| output.status.success())
}

// DECISION: D203
#[cfg(target_os = "linux")]
fn sandbox(
    root: &Path,
    argv: &[String],
    _scratch: &Path,
    confinement: &Confinement,
) -> Result<Command> {
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
    match confinement {
        Confinement::Reading => {}
        Confinement::Writing => {
            command.arg("--bind").arg(root).arg(root);
        }
    }
    command.arg("--chdir").arg(root).arg("--").args(argv);
    Ok(command)
}

// DECISION: D203
#[cfg(target_os = "macos")]
fn sandbox(
    root: &Path,
    argv: &[String],
    scratch: &Path,
    confinement: &Confinement,
) -> Result<Command> {
    let scratch = canonical(scratch)?;
    let subpath = |path: &Path| format!(" (subpath {})", quoted(&path.to_string_lossy()));
    let writable = match confinement {
        Confinement::Reading => subpath(&scratch),
        Confinement::Writing => subpath(&scratch) + &subpath(&canonical(root)?),
    };
    let profile = format!(
        "(version 1)(allow default)(deny network*)(deny file-write*)(deny appleevent-send)(deny lsopen)\
         (allow file-write* (literal \"/dev/null\") (literal \"/dev/zero\") \
         (literal \"/dev/dtracehelper\") (regex #\"^/dev/tty\") (regex #\"^/dev/fd/\"){writable})"
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
