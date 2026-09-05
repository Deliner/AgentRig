use super::super::config::Context;
use anyhow::Result;
use std::{env, fs, path::Path, process::Command};

fn available(program: &str, cwd: &Path) -> bool {
    let executable = |path: &Path| {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
    };
    if program.contains('/') {
        return executable(&cwd.join(program));
    }
    env::var_os("PATH")
        .is_some_and(|paths| env::split_paths(&paths).any(|path| executable(&path.join(program))))
}
pub fn run(context: &Context) -> Result<i32> {
    let mut failed = false;
    println!(
        "runtime {}: compatible; platform {}",
        super::super::config::VERSION,
        env::consts::OS
    );
    let installed = Command::new(context.root.join(".worker/bin/discipline-worker"))
        .arg("--version")
        .output();
    let matches = installed.is_ok_and(|output| {
        output.status.success()
            && String::from_utf8_lossy(&output.stdout).trim()
                == format!("discipline-worker {}", super::super::config::VERSION)
    });
    println!(
        "installed binary: {}",
        if matches {
            "compatible"
        } else {
            "MISSING OR INCOMPATIBLE"
        }
    );
    failed |= !matches;
    for (name, command) in &context.config.commands {
        if let Some(program) = command.argv.first() {
            let found = available(program, &context.path(&command.cwd)?);
            println!(
                "command {name}: {} ({program})",
                if found { "available" } else { "MISSING" }
            );
            failed |= !found;
        }
    }
    let read_only = context
        .config
        .commands
        .values()
        .any(|command| command.read_only);
    if read_only {
        let probe = Command::new("bwrap")
            .args(["--ro-bind", "/", "/", "--unshare-all", "--", "true"])
            .output();
        let works = probe.is_ok_and(|output| output.status.success());
        println!(
            "read-only backend: {}",
            if works { "available" } else { "UNAVAILABLE" }
        );
        failed |= !works;
    }
    let hooks =
        crate::util::git(&context.root, &["config", "--get", "core.hooksPath"]).unwrap_or_default();
    let registered = hooks == ".worker/hooks"
        && super::adapters::git_hooks().iter().all(|(path, contents)| {
            available(path, &context.root)
                && fs::read(context.root.join(path)).is_ok_and(|actual| actual == *contents)
        });
    println!(
        "Git hooks: {}",
        if registered {
            "registered"
        } else {
            "NOT REGISTERED OR CHANGED; verify core.hooksPath, adapter contents and executable permissions"
        }
    );
    failed |= !registered;
    let settings = fs::read_to_string(context.root.join(".codex/config.toml"))
        .ok()
        .and_then(|text| toml::from_str::<toml::Value>(&text).ok());
    let enabled = settings
        .as_ref()
        .and_then(|value| value.get("features"))
        .and_then(|value| value.get("hooks"))
        .and_then(toml::Value::as_bool)
        == Some(true);
    let configured = fs::read(context.root.join(".codex/hooks.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let expected: serde_json::Value = serde_json::from_slice(&super::adapters::registration()?)?;
    let codex = enabled
        && configured.as_ref().is_some_and(|value| {
            ["SessionStart", "PreToolUse"].iter().all(|event| {
                let desired = &expected["hooks"][event][0];
                value["hooks"][event]
                    .as_array()
                    .is_some_and(|routes| routes.contains(desired))
            })
        });
    println!(
        "Codex registration: {}",
        if codex {
            "registered"
        } else {
            "MISSING OR CHANGED; inspect .codex/config.toml and .codex/hooks.json"
        }
    );
    failed |= !codex;
    Ok(i32::from(failed))
}
