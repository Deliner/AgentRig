mod configuration;
#[cfg(test)]
mod tests;

use super::config::Profile;
use anyhow::{Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub struct Layout {
    pub input: PathBuf,
    pub private: PathBuf,
    pub codex: PathBuf,
}

pub fn prepare(layout: &Layout, profile: &Profile) -> Result<()> {
    fs::create_dir(&layout.private)?;
    for name in ["work", "codex"] {
        fs::create_dir(layout.private.join(name))?;
    }
    configuration::write(layout, profile)
}

pub fn command(layout: &Layout, profile: &Profile) -> Result<Command> {
    let host = layout.codex.with_file_name("codex-code-mode-host");
    ensure!(host.is_file(), "codex-code-mode-host must be next to Codex");
    let mut command = Command::new("/usr/bin/bwrap");
    command.env_clear().args([
        "--die-with-parent",
        "--new-session",
        "--unshare-user",
        "--unshare-pid",
        "--unshare-ipc",
        "--unshare-uts",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/tmp",
        "--dir",
        "/home/delegate",
        "--chdir",
        "/work",
    ]);
    review_runner::execution::sandbox::system_libraries(&mut command);
    mounts(&mut command, layout, profile);
    command
        .args(["--ro-bind"])
        .arg(&host)
        .arg("/codex-code-mode-host");
    configuration::environment(&mut command, profile)?;
    command.args([
        "/codex-cli", "exec", "--ignore-rules", "--ephemeral", "--skip-git-repo-check",
        "--dangerously-bypass-approvals-and-sandbox", "--json", "--output-schema",
        "/delegate-input/schema.json", "--output-last-message", "/work/result.json",
        "Read /delegate-input/prompt.md and /delegate-input/request.json. Complete the task within its contract. Inputs are in /project and /inputs. Write required artifacts under /work and return the contracted JSON response.",
    ]);
    Ok(command)
}

fn mounts(command: &mut Command, layout: &Layout, profile: &Profile) {
    for (source, target, writable) in [
        (layout.input.clone(), "/delegate-input", false),
        (layout.input.join("project"), "/project", false),
        (layout.input.join("inputs"), "/inputs", false),
        (layout.private.join("work"), "/work", true),
        (layout.private.join("codex"), "/codex", true),
        (layout.codex.clone(), "/codex-cli", false),
    ] {
        command
            .arg(if writable { "--bind" } else { "--ro-bind" })
            .arg(source)
            .arg(target);
    }
    command
        .arg("--ro-bind")
        .arg(layout.private.join("codex/config.toml"))
        .arg("/codex/config.toml");
    tools(command, profile);
}

fn tools(command: &mut Command, profile: &Profile) {
    for name in ["sh", "bash", "env"] {
        command
            .args(["--ro-bind"])
            .arg(format!("/usr/bin/{name}"))
            .arg(format!("/bin/{name}"));
    }
    for (name, source) in &profile.programs {
        command
            .arg("--ro-bind")
            .arg(source)
            .arg(format!("/tools/{name}"));
    }
    for source in &profile.skills {
        let name = source.file_name().unwrap().to_string_lossy();
        command
            .arg("--ro-bind")
            .arg(source)
            .arg(format!("/codex/skills/{name}"));
    }
}

pub fn write_prompt(input: &Path, profile: &Profile) -> Result<()> {
    let mut prompt = fs::read_to_string(&profile.prompt)?;
    for skill in &profile.skills {
        let name = skill.file_name().unwrap().to_string_lossy();
        prompt.push_str(&format!(
            "\nApply the configured skill /codex/skills/{name}/SKILL.md.\n"
        ));
    }
    fs::write(input.join("prompt.md"), prompt)?;
    Ok(())
}
