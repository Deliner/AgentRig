use crate::command_runner;
use crate::project_config::{Access, Context};
use crate::project_setup::bundle;
use crate::project_setup::rust::Requirement;
use crate::shared::calling_agent::Agent;
use crate::shared::console;
use crate::shared::fs_with_context::{self as fs, PermissionsExt};
use anyhow::Result;
use std::{env, path::Path};

fn available(program: &str, cwd: &Path) -> bool {
    let executable = |path: &Path| {
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
    console::stdout(format!(
        "runtime {version}: compatible; platform {platform}",
        version = crate::project_config::VERSION,
        platform = env::consts::OS
    ));
    let mut failed = !installed_binary(context);
    failed |= !command_availability(context)?;
    failed |= !rust_toolchain(context)?;
    failed |= !sandbox_availability(context);
    failed |= !vcs_registration(context)?;
    failed |= !codex_registration(context)?;
    failed |= !bundle::claude_registration(context)?;
    if failed {
        let rerun = crate::shared::diagnostics::rerun(&context.root, &["doctor".into()]);
        console::stderr(crate::shared::diagnostics::Guidance {
            level: "ERROR",
            id: "doctor",
            location: &context.root.display().to_string(),
            message: "installation checks failed; see the individual results above",
            skill: &context.caller.skill(&context.config.config_skill),
            rerun: &rerun,
        });
    }
    Ok(i32::from(failed))
}
fn installed_binary(context: &Context) -> bool {
    let binary = context
        .root
        .join(context.config.paths.service_path("bin/agentrig"));
    let installed = command_runner::tracked(
        &context.root,
        &[binary.to_string_lossy().into_owned(), "--version".into()],
        command_runner::Sandboxing {
            access: Access::Unconfined,
            capture: command_runner::Capture::Collect,
            cargo_target: None,
            rustflags: &context.config.rust.rustflags,
        },
        None,
    );
    let matches = installed.is_ok_and(|output| {
        output.status.success()
            && String::from_utf8_lossy(&output.stdout).trim()
                == format!("agentrig {}", crate::project_config::VERSION)
    });
    console::stdout(format!(
        "installed binary: {}",
        if matches {
            "compatible"
        } else {
            "MISSING OR INCOMPATIBLE"
        }
    ));
    matches
}
fn command_availability(context: &Context) -> Result<bool> {
    let mut failed = false;
    for (name, command) in &context.config.commands {
        if let Some(program) = command.argv.first() {
            let found = available(program, &context.path(&command.cwd)?);
            console::stdout(format!(
                "command {name}: {} ({program})",
                if found { "available" } else { "MISSING" }
            ));
            failed |= !found;
        }
    }
    Ok(!failed)
}
// DECISION: D174
fn rust_toolchain(context: &Context) -> Result<bool> {
    let cargo = context.config.commands.values().any(|command| {
        let program = command.argv.first().map(Path::new);
        program
            .and_then(Path::file_name)
            .is_some_and(|name| name == "cargo")
    });
    let without_cargo = !cargo;
    if without_cargo {
        return Ok(true);
    }
    let toolchain = Requirement::inspect(&context.root)?;
    let met = toolchain.met();
    let status = if met { "" } else { "OLDER OR MISSING, " };
    console::stdout(format!("rust toolchain: {status}{toolchain}"));
    Ok(met)
}
fn sandbox_availability(context: &Context) -> bool {
    let mut failed = false;
    let sandboxed = context
        .config
        .commands
        .values()
        .any(|command| command.access.sandboxed());
    if sandboxed {
        let works = command_runner::sandbox_available();
        console::stdout(format!(
            "sandbox backend: {}",
            if works { "available" } else { "UNAVAILABLE" }
        ));
        failed |= !works;
    }
    !failed
}
fn vcs_registration(context: &Context) -> Result<bool> {
    let registered = crate::shared::vcs::Repository::new(&context.root)
        .hooks_registered(&context.config.paths.service_path("hooks"))?;
    let registered = registered
        && bundle::vcs_hooks(&context.config)?
            .iter()
            .all(|(path, contents)| {
                available(path, &context.root)
                    && fs::read(context.root.join(path)).is_ok_and(|actual| actual == *contents)
            });
    console::stdout(format!(
        "git hooks: {}",
        if registered {
            "registered"
        } else {
            "NOT REGISTERED OR CHANGED; verify VCS registration, adapter contents and executable permissions"
        }
    ));
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
    let codex =
        enabled && bundle::hooks_registered(&context.config, configured.as_ref(), Agent::Codex)?;
    console::stdout(format!(
        "Codex registration: {}",
        if codex {
            "registered"
        } else {
            "MISSING OR CHANGED; inspect .codex/config.toml and .codex/hooks.json"
        }
    ));
    Ok(codex)
}
