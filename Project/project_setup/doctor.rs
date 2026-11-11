use crate::project_config::{Access, Context};
use crate::shared::console;
use crate::shared::fs::{self, PermissionsExt};
use crate::structural_lint::Location;
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
    failed |= !sandbox_availability(context);
    failed |= !vcs_registration(context)?;
    failed |= !match context.config.frontend {
        crate::project_config::Frontend::Codex => codex_registration(context)?,
        crate::project_config::Frontend::ClaudeCode => super::claude::registration(context)?,
    };
    if failed {
        let rerun = crate::shared::diagnostics::rerun(&context.root, &["doctor".into()]);
        console::stderr(crate::shared::diagnostics::Guidance {
            level: "ERROR",
            id: "doctor",
            location: &context.root.display().to_string(),
            message: "installation checks failed; see the individual results above",
            skill: &context.config.config_skill,
            rerun: &rerun,
        });
    }
    Ok(i32::from(failed))
}
fn installed_binary(context: &Context) -> bool {
    let binary = context
        .root
        .join(context.config.paths.service_path("bin/agentrig"));
    let installed = crate::command_runner::sandbox::tracked(
        &context.root,
        &[binary.to_string_lossy().into_owned(), "--version".into()],
        crate::command_runner::sandbox::Sandboxing {
            access: Access::Writable,
            capture: crate::command_runner::jobs::Capture::Collect,
            cargo_target: None,
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
fn sandbox_availability(context: &Context) -> bool {
    let mut failed = false;
    let read_only = context
        .config
        .commands
        .values()
        .any(|command| command.access.read_only());
    if read_only {
        let works = crate::command_runner::sandbox::available();
        console::stdout(format!(
            "read-only backend: {}",
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
        && super::adapters::vcs_hooks(&context.config)?
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
    let codex = enabled && hooks_registered(context, configured.as_ref())?;
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

pub(super) fn hooks_registered(
    context: &Context,
    configured: Option<&serde_json::Value>,
) -> Result<bool> {
    let expected: serde_json::Value = serde_json::from_slice(&super::adapters::registration(
        &context.config,
        &super::adapters::generated(&context.config)?.root_command,
    )?)?;
    Ok(configured.is_some_and(|value| {
        expected["hooks"].as_object().is_some_and(|hooks| {
            hooks.iter().all(|(event, desired)| {
                value["hooks"][event].as_array().is_some_and(|routes| {
                    desired
                        .as_array()
                        .is_some_and(|items| items.iter().all(|route| routes.contains(route)))
                })
            })
        })
    }))
}
pub fn config_check(context: &Context) -> Result<i32> {
    let lint = if context.config.capabilities.lint {
        let config = context.path(&context.config.paths.lint)?;
        crate::structural_lint::run(
            Location {
                root: &context.root,
                config: &config,
            },
            crate::structural_lint::Format::Json,
            crate::structural_lint::Mode::ConfigCheck,
        )?
    } else {
        0
    };
    let uncovered = super::coverage::report(context)?;
    let linted = lint == 0;
    let code = if linted { i32::from(uncovered) } else { lint };
    let valid = code == 0;
    if valid {
        console::stdout(format!(
            "Scaffold configuration is valid (runtime {})",
            crate::project_config::VERSION
        ));
    }
    Ok(code)
}
