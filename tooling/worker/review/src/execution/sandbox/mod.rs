use crate::config::Reviewer;
mod claude;
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
    pub executable: PathBuf,
}
pub fn prepare(role: &Path, reviewer: &Reviewer) -> Result<()> {
    for directory in ["work", "bin", client_directory(&reviewer.frontend)] {
        fs::create_dir_all(role.join(directory))?;
    }
    fs::write(role.join("bin/review-runner"), [])?;
    let is_claude = reviewer.frontend == "claude-code";
    if is_claude {
        return claude::prepare(role);
    }
    let explicit = &reviewer.credentials.codex_auth_file_env;
    if let Some(reference) = explicit {
        let source = env::var_os(reference)
            .with_context(|| format!("missing credential reference {reference}"))?;
        return copy_auth(&PathBuf::from(source), role);
    }
    let api_key = reviewer.credentials.env.contains_key("OPENAI_API_KEY");
    if api_key {
        return Ok(());
    }
    let home = env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".codex")))
        .context("provide CODEX_HOME with isolated CLI authentication")?;
    copy_auth(&home.join("auth.json"), role)
}
fn copy_auth(source: &Path, role: &Path) -> Result<()> {
    let target = role.join("codex/auth.json");
    fs::copy(source, &target)
        .context("provide Codex authentication separately in CODEX_HOME/auth.json")?;
    fs::set_permissions(target, fs::Permissions::from_mode(0o600))?;
    Ok(())
}
pub fn executable(frontend: &str) -> Result<PathBuf> {
    match frontend {
        "codex" => native_codex(),
        "claude-code" => native_claude_from("REVIEW_CLAUDE_BIN"),
        _ => anyhow::bail!("unsupported review frontend {frontend}"),
    }
}
pub fn native_claude_from(variable: &str) -> Result<PathBuf> {
    claude::executable(variable)
}
fn client_directory(frontend: &str) -> &str {
    match frontend {
        "claude-code" => "claude",
        _ => "codex",
    }
}
pub fn native_codex() -> Result<PathBuf> {
    native_codex_from("REVIEW_CODEX_BIN")
}
pub fn native_codex_from(variable: &str) -> Result<PathBuf> {
    if let Some(path) = env::var_os(variable) {
        return Ok(PathBuf::from(path).canonicalize()?);
    }
    let path = env::var_os("PATH").context("PATH is required to find codex")?;
    let executable = env::split_paths(&path)
        .map(|directory| directory.join("codex"))
        .find(|path| path.is_file())
        .with_context(|| format!("codex is not installed; set {variable}"))?
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
        anyhow::bail!("native codex binary not found; set {variable}");
    }
    Ok(executable)
}
pub fn command(layout: &Layout, reviewer: &Reviewer) -> Result<Command> {
    let mut command = Command::new("bwrap");
    command
        .env_clear()
        .envs(crate::config::credentials::resolve_references(
            &reviewer.credentials.env,
        )?);
    command.args([
        "--die-with-parent",
        "--new-session",
        "--unshare-user",
        "--unshare-pid",
        "--unshare-ipc",
        "--unshare-uts",
    ]);
    runtime_mounts(&mut command);
    review_mounts(&mut command, layout, &reviewer.frontend);
    command
        .arg("--ro-bind")
        .arg(env::current_exe()?)
        .arg("/review-bin/review-runner");
    environment(&mut command);
    let is_claude = reviewer.frontend == "claude-code";
    if is_claude {
        claude::command(&mut command, layout, reviewer);
    } else {
        codex(&mut command, layout, reviewer)?;
    }
    Ok(command)
}
fn codex(command: &mut Command, layout: &Layout, reviewer: &Reviewer) -> Result<()> {
    let host = layout.executable.with_file_name("codex-code-mode-host");
    ensure!(
        host.is_file(),
        "codex-code-mode-host must be next to native codex"
    );
    command
        .arg("--ro-bind")
        .arg(&layout.executable)
        .arg("/codex-cli")
        .arg("--ro-bind")
        .arg(host)
        .arg("/codex-code-mode-host")
        .args(["--setenv", "CODEX_HOME", "/codex", "/codex-cli", "exec"]);
    cli(command, reviewer);
    Ok(())
}
fn environment(command: &mut Command) {
    command.args([
        "--setenv",
        "HOME",
        "/home/critic",
        "--setenv",
        "PATH",
        "/usr/bin:/bin",
        "--chdir",
        "/work",
    ]);
}
fn review_mounts(command: &mut Command, layout: &Layout, frontend: &str) {
    let client = client_directory(frontend);
    let home = format!("/{client}");
    for (source, target, writable) in [
        (layout.project.to_path_buf(), "/project", false),
        (layout.input.to_path_buf(), "/review-input", false),
        (layout.role.join("work"), "/work", true),
        (layout.role.join("bin"), "/review-bin", false),
        (layout.role.join(client), home.as_str(), true),
    ] {
        command
            .arg(if writable { "--bind" } else { "--ro-bind" })
            .arg(source)
            .arg(target);
    }
}
fn runtime_mounts(command: &mut Command) {
    command.args(["--ro-bind", "/usr/bin", "/usr/bin"]);
    system_libraries(command);
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
pub fn system_libraries(command: &mut Command) {
    for path in [
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
}
fn cli(command: &mut Command, reviewer: &Reviewer) {
    command.args(["--ignore-user-config", "--ignore-rules", "--ephemeral", "--skip-git-repo-check", "--dangerously-bypass-approvals-and-sandbox", "--dangerously-bypass-hook-trust", "--json", "-m", &reviewer.model, "-c"])
        .arg(format!("model_reasoning_effort={:?}", reviewer.reasoning_effort))
        .args(["-c", "features.hooks=true", "-c", "hooks.Stop=[{hooks=[{type=\"command\",command=\"/review-bin/review-runner review-hook\",timeout=10}]}]", "-"]);
}
