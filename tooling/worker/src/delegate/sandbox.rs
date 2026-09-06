mod configuration;
mod resources;
#[cfg(test)]
mod tests;

use super::config::{Mode, Profile};
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

pub fn prepare(layout: &Layout, profile: &Profile) -> Result<serde_json::Value> {
    fs::create_dir(&layout.private)?;
    for name in ["work", "codex"] {
        fs::create_dir(layout.private.join(name))?;
    }
    configuration::write(layout, profile)?;
    let coding = matches!(profile.mode, Mode::Code);
    if coding {
        super::code::Workspace::prepare(
            &layout.input.join("project"),
            &layout.private.join("code"),
        )?;
    }
    resources::prepare(layout, profile)
}

pub fn command(layout: &Layout, profile: &Profile) -> Result<Command> {
    let mut command = base(layout, profile)?;
    executor(&mut command, profile);
    Ok(command)
}

pub fn check(layout: &Layout, profile: &Profile, argv: &[String]) -> Result<Command> {
    let mut command = base(layout, profile)?;
    command
        .arg("--ro-bind")
        .arg(layout.private.join("code/project"))
        .arg("/project")
        .args(["--chdir", "/project"])
        .arg(format!("/tools/{}", argv[0]))
        .args(&argv[1..]);
    Ok(command)
}

fn base(layout: &Layout, profile: &Profile) -> Result<Command> {
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
    Ok(command)
}

fn executor(command: &mut Command, profile: &Profile) {
    command.args([
        "/codex-cli",
        "exec",
        "--ignore-rules",
        "--ephemeral",
        "--skip-git-repo-check",
        "--dangerously-bypass-approvals-and-sandbox",
        "--json",
        "--output-schema",
        "/delegate-input/schema.json",
        "--output-last-message",
        "/work/result.json",
    ]);
    let configured_hooks = !profile.hooks.is_empty();
    if configured_hooks {
        command.arg("--dangerously-bypass-hook-trust");
    }
    command.arg(instructions(profile));
}

fn instructions(profile: &Profile) -> String {
    let programs = profile
        .programs
        .keys()
        .map(|name| format!("/tools/{name}"))
        .collect::<Vec<_>>()
        .join(", ");
    let coding = matches!(profile.mode, Mode::Code);
    let editing = if coding {
        " Edit code under /project within contract.changes.write_paths. Git metadata is runner-owned; use file edits. Checks run later with /project read-only; put build outputs under /work or /tmp."
    } else {
        " /project is read-only."
    };
    format!(
        "This is a minimal sandbox. Available programs: /bin/bash, /bin/sh, /bin/env, {programs}. Use these absolute paths or shell builtins; other host utilities are unavailable. Read /delegate-input/prompt.md and /delegate-input/request.json. Complete the task within its contract. Inputs are in /project and /inputs. Write required artifacts under /work and return the contracted JSON response.{editing}"
    )
}

fn mounts(command: &mut Command, layout: &Layout, profile: &Profile) {
    let coding = matches!(profile.mode, Mode::Code);
    let project = if coding {
        layout.private.join("code/project")
    } else {
        layout.input.join("project")
    };
    for (source, target, writable) in [
        (layout.input.clone(), "/delegate-input", false),
        (project, "/project", coding),
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
    tools(command, layout, profile);
}

fn tools(command: &mut Command, layout: &Layout, profile: &Profile) {
    for name in ["sh", "bash", "env"] {
        command
            .args(["--ro-bind"])
            .arg(format!("/usr/bin/{name}"))
            .arg(format!("/bin/{name}"));
    }
    for name in profile.programs.keys() {
        command
            .arg("--ro-bind")
            .arg(layout.private.join(format!("environment/programs/{name}")))
            .arg(format!("/tools/{name}"));
    }
    for source in &profile.skills {
        let name = source.file_name().unwrap().to_string_lossy();
        command
            .arg("--ro-bind")
            .arg(layout.private.join(format!("environment/skills/{name}")))
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
