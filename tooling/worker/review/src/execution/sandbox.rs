use crate::config::Reviewer;
use anyhow::{Context, Result, ensure};
use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

pub struct Layout {
    pub project: PathBuf,
    pub input: PathBuf,
    pub role: PathBuf,
    pub codex: PathBuf,
}
pub fn prepare(role: &Path) -> Result<()> {
    for directory in ["work", "bin", "codex"] {
        fs::create_dir_all(role.join(directory))?;
    }
    fs::write(role.join("bin/review-runner"), [])?;
    let home = env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".codex")))
        .context("provide CODEX_HOME with isolated CLI authentication")?;
    let target = role.join("codex/auth.json");
    fs::copy(home.join("auth.json"), &target)
        .context("provide Codex authentication separately in CODEX_HOME/auth.json")?;
    fs::set_permissions(target, fs::Permissions::from_mode(0o600))?;
    Ok(())
}
pub fn native_codex() -> Result<PathBuf> {
    if let Some(path) = env::var_os("REVIEW_CODEX_BIN") {
        return Ok(PathBuf::from(path).canonicalize()?);
    }
    let path = env::var_os("PATH").context("PATH is required to find codex")?;
    let executable = env::split_paths(&path)
        .map(|directory| directory.join("codex"))
        .find(|path| path.is_file())
        .context("codex is not installed; set REVIEW_CODEX_BIN")?
        .canonicalize()?;
    let wrapper = executable
        .extension()
        .is_some_and(|extension| extension == "js");
    if wrapper {
        let modules = executable
            .parent()
            .and_then(Path::parent)
            .context("Codex package directory")?
            .join("node_modules/@openai");
        for package in fs::read_dir(modules)? {
            let vendor = package?.path().join("vendor");
            let exists = vendor.is_dir();
            if exists {
                for platform in fs::read_dir(vendor)? {
                    let binary = platform?.path().join("bin/codex");
                    let installed = binary.is_file();
                    if installed {
                        return Ok(binary);
                    }
                }
            }
        }
        anyhow::bail!("native codex binary not found; set REVIEW_CODEX_BIN");
    }
    Ok(executable)
}
pub fn command(layout: &Layout, reviewer: &Reviewer) -> Result<Command> {
    let host = layout.codex.with_file_name("codex-code-mode-host");
    ensure!(
        host.is_file(),
        "codex-code-mode-host must be next to native codex"
    );
    let mut command = Command::new("bwrap");
    command.args([
        "--die-with-parent",
        "--new-session",
        "--unshare-user",
        "--unshare-pid",
        "--unshare-ipc",
        "--unshare-uts",
        "--clearenv",
    ]);
    runtime_mounts(&mut command);
    review_mounts(&mut command, layout);
    command
        .arg("--ro-bind")
        .arg(env::current_exe()?)
        .arg("/review-bin/review-runner")
        .arg("--ro-bind")
        .arg(&layout.codex)
        .arg("/codex-cli")
        .arg("--ro-bind")
        .arg(host)
        .arg("/codex-code-mode-host");
    environment(&mut command);
    cli(&mut command, reviewer);
    Ok(command)
}
fn environment(command: &mut Command) {
    command.args([
        "--setenv",
        "HOME",
        "/home/critic",
        "--setenv",
        "CODEX_HOME",
        "/codex",
        "--setenv",
        "PATH",
        "/usr/bin:/bin",
        "--chdir",
        "/work",
        "/codex-cli",
        "exec",
    ]);
}
fn review_mounts(command: &mut Command, layout: &Layout) {
    for (source, target, writable) in [
        (layout.project.to_path_buf(), "/project", false),
        (layout.input.to_path_buf(), "/review-input", false),
        (layout.role.join("work"), "/work", true),
        (layout.role.join("bin"), "/review-bin", false),
        (layout.role.join("codex"), "/codex", true),
    ] {
        command
            .arg(if writable { "--bind" } else { "--ro-bind" })
            .arg(source)
            .arg(target);
    }
}
fn runtime_mounts(command: &mut Command) {
    for path in [
        "/usr/bin",
        "/usr/lib",
        "/usr/lib64",
        "/lib",
        "/lib64",
        "/etc/ssl",
        "/etc/resolv.conf",
        "/etc/hosts",
        "/etc/nsswitch.conf",
    ] {
        let exists = Path::new(path).exists();
        if exists {
            command.args(["--ro-bind", path, path]);
        }
    }
    command.args([
        "--symlink",
        "usr/bin",
        "/bin",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/tmp",
        "--dir",
        "/home/critic",
    ]);
}
fn cli(command: &mut Command, reviewer: &Reviewer) {
    command.args(["--ignore-user-config", "--ignore-rules", "--ephemeral", "--skip-git-repo-check", "--dangerously-bypass-approvals-and-sandbox", "--dangerously-bypass-hook-trust", "--json", "-m", &reviewer.model, "-c"])
        .arg(format!("model_reasoning_effort={:?}", reviewer.reasoning_effort))
        .args(["-c", "features.hooks=true", "-c", "hooks.Stop=[{hooks=[{type=\"command\",command=\"/review-bin/review-runner review-hook\",timeout=10}]}]", "-"]);
}
