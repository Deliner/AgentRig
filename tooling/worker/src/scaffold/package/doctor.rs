use super::super::config::Context;
use anyhow::Result;
use std::{env, fs, path::Path, process::Command};

fn available(program: &str, cwd: &Path) -> bool {
    let executable = |path: &Path| {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
    };
    let explicit_path = program.contains('/');
    if explicit_path {
        return executable(&cwd.join(program));
    }
    env::var_os("PATH")
        .is_some_and(|paths| env::split_paths(&paths).any(|path| executable(&path.join(program))))
}
pub fn run(context: &Context) -> Result<i32> {
    println!(
        "runtime {}: compatible; platform {}",
        super::super::config::VERSION,
        env::consts::OS
    );
    let mut failed = !installed_binary(context);
    failed |= !command_availability(context)?;
    failed |= !sandbox_availability(context);
    failed |= !review_dependencies(context);
    failed |= !git_registration(context);
    failed |= !codex_registration(context)?;
    if failed {
        let rerun = crate::diagnostics::rerun(&context.root, &["doctor".into()]);
        eprintln!(
            "{}",
            crate::diagnostics::Guidance {
                level: "ERROR",
                id: "doctor",
                location: &context.root.display().to_string(),
                message: "installation checks failed; see the individual results above",
                skill: &context.config.config_skill,
                rerun: &rerun,
            }
        );
    }
    Ok(i32::from(failed))
}
fn installed_binary(context: &Context) -> bool {
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
    matches
}
fn command_availability(context: &Context) -> Result<bool> {
    let mut failed = false;
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
    Ok(!failed)
}
fn sandbox_availability(context: &Context) -> bool {
    let mut failed = false;
    let read_only = context
        .config
        .commands
        .values()
        .any(|command| command.read_only)
        || context.config.capabilities.review.is_some();
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
    !failed
}
fn git_registration(context: &Context) -> bool {
    let hooks =
        crate::util::git(&context.root, &["config", "--get", "core.hooksPath"]).unwrap_or_default();
    let registered = hooks == ".worker/hooks"
        && super::adapters::git_hooks().iter().all(|(path, contents)| {
            available(path, &context.root)
                && fs::read(context.root.join(path)).is_ok_and(|actual| {
                    actual == *contents || super::manifest::approved(&context.root, path, &actual)
                })
        });
    println!(
        "Git hooks: {}",
        if registered {
            "registered"
        } else {
            "NOT REGISTERED OR CHANGED; verify core.hooksPath, adapter contents and executable permissions"
        }
    );
    registered
}
fn codex_registration(context: &Context) -> Result<bool> {
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
    Ok(codex)
}

fn review_dependencies(context: &Context) -> bool {
    let enabled = context.config.capabilities.review.is_some();
    if enabled {
        let native = review_runner::execution::sandbox::native_codex();
        let available = native.as_ref().is_ok_and(|path| {
            available(&path.to_string_lossy(), &context.root)
                && path.with_file_name("codex-code-mode-host").is_file()
        });
        println!(
            "review Codex and code-mode host: {}",
            if available {
                "available"
            } else {
                "MISSING; install Codex or set REVIEW_CODEX_BIN"
            }
        );
        return available;
    }
    true
}
