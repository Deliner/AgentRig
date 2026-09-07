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
    let capability = agentrig::jobs::capability();
    println!("process scope capability: {capability}");
    let required = context.config.processes.foreground
        == super::super::config::Containment::Systemd
        || context.config.capabilities.delegation.is_some();
    failed |= required && capability["available"] != true;
    failed |= !command_availability(context)?;
    failed |= !sandbox_availability(context);
    failed |= !executor_dependencies(context);
    failed |= !vcs_registration(context)?;
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
    let installed = Command::new(
        context
            .root
            .join(context.config.paths.service_path("bin/agentrig")),
    )
    .arg("--version")
    .output();
    let matches = installed.is_ok_and(|output| {
        output.status.success()
            && String::from_utf8_lossy(&output.stdout).trim()
                == format!("agentrig {}", super::super::config::VERSION)
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
        || context.config.vcs.backend != review_runner::vcs::Kind::Git.into()
        || context.config.capabilities.review.is_some()
        || context.config.capabilities.delegation.is_some();
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
fn vcs_registration(context: &Context) -> Result<bool> {
    let repository = context.config.vcs.backend.repository(&context.root)?;
    let registered = match repository {
        Some(repository) => {
            repository.hooks_registered(&context.config.paths.service_path("hooks"))?
        }
        None => false,
    };
    let registered = registered
        && super::adapters::vcs_hooks(&context.config)?
            .iter()
            .all(|(path, contents)| {
                available(path, &context.root)
                    && fs::read(context.root.join(path)).is_ok_and(|actual| {
                        actual == *contents || super::manifest::approved(context, path, &actual)
                    })
            });
    println!(
        "{} hooks: {}",
        context.config.vcs.backend.executable(),
        if registered {
            "registered"
        } else {
            "NOT REGISTERED OR CHANGED; verify VCS registration, adapter contents and executable permissions"
        }
    );
    Ok(registered)
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
    let codex = enabled && hooks_registered(context, configured.as_ref())?;
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

fn hooks_registered(context: &Context, configured: Option<&serde_json::Value>) -> Result<bool> {
    let expected: serde_json::Value =
        serde_json::from_slice(&super::adapters::registration(&context.config)?)?;
    Ok(configured.is_some_and(|value| {
        expected["hooks"]
            .as_object()
            .unwrap()
            .iter()
            .all(|(event, desired)| {
                value["hooks"][event].as_array().is_some_and(|routes| {
                    desired
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|route| routes.contains(route))
                })
            })
    }))
}

fn executor_dependencies(context: &Context) -> bool {
    let mut success = true;
    for (capability, variable) in [
        (&context.config.capabilities.review, "REVIEW_CODEX_BIN"),
        (
            &context.config.capabilities.delegation,
            "DELEGATE_CODEX_BIN",
        ),
    ] {
        let enabled = capability.is_some();
        if enabled {
            success &= native_executor(context, variable);
        }
    }
    success
}
fn native_executor(context: &Context, variable: &str) -> bool {
    let native = review_runner::execution::sandbox::native_codex_from(variable);
    let available = native.as_ref().is_ok_and(|path| {
        available(&path.to_string_lossy(), &context.root)
            && path.with_file_name("codex-code-mode-host").is_file()
    });
    println!(
        "{variable} Codex and code-mode host: {}",
        if available {
            "available"
        } else {
            "MISSING; install Codex or set the indicated variable"
        }
    );
    available
}
