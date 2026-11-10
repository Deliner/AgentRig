use super::Command;
use crate::{
    change_delivery::git,
    command_runner,
    project_config::Context,
    project_memory::{self, evidence},
    project_setup,
    shared::{arguments::forwarded, console, diagnostics},
};
use anyhow::{Result, bail};

pub(super) fn configured_command(
    context: &Context,
    command: Command,
    args: &[String],
) -> Result<i32> {
    let root = &context.root;
    let git = &context.config.git;
    match command {
        Command::Doctor => project_setup::doctor(context),
        Command::RuntimeAdd => project_setup::runtime_add(context),
        Command::ConfigCheck => project_setup::config_check(context),
        Command::GuardReference => {
            git::guard_reference_with(root, args.first().map(String::as_str).unwrap_or(""), git)
        }
        Command::FeatureStart => git::start(context, &args[0]),
        Command::FeatureMerge => git::merge(context),
        Command::MemoryCheck => project_memory::check(context),
        Command::Resume => {
            project_memory::resume(context)?;
            Ok(0)
        }
        Command::Commands => {
            command_runner::registry::list(context)?;
            Ok(0)
        }
        Command::Run => {
            let (name, extra) = args
                .split_first()
                .ok_or_else(|| anyhow::anyhow!("run COMMAND [-- ARGS]"))?;
            run_command(context, name, forwarded(extra))
        }
        Command::Report => {
            command_runner::report(context)?;
            evidence::report(context)?;
            Ok(0)
        }
        _ => bail!("unknown scaffold command {}", command.name()),
    }
}

fn run_command(context: &Context, name: &str, extra: &[String]) -> Result<i32> {
    let code = command_runner::run(context, name, extra)?;
    let failed = code != 0;
    let check = context
        .config
        .checks
        .iter()
        .find(|check| check.command.as_deref() == Some(name));
    if let Some(check) = check.filter(|_| failed) {
        let mut args = vec!["run".into(), name.into(), "--".into()];
        args.extend_from_slice(extra);
        let rerun = diagnostics::rerun(&context.root, &args);
        console::stderr(diagnostics::Guidance {
            level: "ERROR",
            id: &check.id,
            location: &context.config.commands[name].cwd,
            message: &format!("exited {code}; see original output above"),
            skill: &check.skill,
            rerun: &rerun,
        });
    }
    Ok(code)
}
