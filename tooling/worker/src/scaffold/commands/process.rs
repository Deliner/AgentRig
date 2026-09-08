pub use crate::jobs::process::exit_code;
use anyhow::{Result, ensure};
use std::{
    path::Path,
    process::{Command, Output},
};

pub fn run(root: &Path, argv: &[String], read_only: bool) -> Result<i32> {
    Ok(exit_code(&execute(root, argv, read_only, false)?))
}
pub fn execute(root: &Path, argv: &[String], read_only: bool, capture: bool) -> Result<Output> {
    tracked(root, argv, (read_only, capture), None)
}
pub fn tracked(
    root: &Path,
    argv: &[String],
    options: (bool, bool),
    job: Option<&mut crate::jobs::Job>,
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
    crate::jobs::process::execute(&mut command, capture, job)
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
