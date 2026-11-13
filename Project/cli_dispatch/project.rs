use super::Configured;
use crate::{
    change_delivery::git,
    command_runner,
    project_config::Context,
    project_memory::{self, check_evidence},
    project_setup,
    shared::{console, diagnostics, shared_options::forwarded},
};
use anyhow::Result;

pub(super) fn configured_command(
    context: &Context,
    command: Configured,
    args: &[String],
) -> Result<i32> {
    let root = &context.root;
    let git = &context.config.git;
    match command {
        Configured::Doctor => project_setup::doctor(context),
        Configured::RuntimeAdd => project_setup::runtime_add(context),
        Configured::ConfigCheck => project_setup::config_check(context),
        Configured::GuardReference => {
            git::guard_reference_with(root, args.first().map(String::as_str).unwrap_or(""), git)
        }
        Configured::FeatureStart => git::start(context, &args[0]),
        Configured::FeatureMerge => git::merge(context),
        Configured::FeaturePush => git::push(context),
        Configured::MemoryCheck => project_memory::check(context),
        Configured::Resume => {
            project_memory::resume(context)?;
            Ok(0)
        }
        Configured::Commands => {
            command_runner::tool_registry::list(context)?;
            Ok(0)
        }
        Configured::Run => {
            let (name, extra) = args
                .split_first()
                .ok_or_else(|| anyhow::anyhow!("run COMMAND [-- ARGS]"))?;
            run_command(context, name, forwarded(extra))
        }
        Configured::Report => {
            command_runner::report(context)?;
            check_evidence::report(context)?;
            Ok(0)
        }
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
