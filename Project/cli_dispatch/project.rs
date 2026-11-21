use super::Configured;
use crate::{
    change_delivery, command_runner,
    project_config::Context,
    project_memory, project_setup,
    shared::{console, diagnostics, shared_options::forwarded},
};
use anyhow::Result;

pub(super) fn configured_command(
    context: &Context,
    command: Configured,
    args: &[String],
) -> Result<i32> {
    match command {
        Configured::Doctor => project_setup::doctor(context),
        Configured::RuntimeAdd => project_setup::runtime_add(context),
        Configured::ConfigCheck => project_setup::config_check(context),
        Configured::GuardReference => {
            let phase = args.first().map(String::as_str).unwrap_or("");
            change_delivery::guard_reference_with(&context.root, phase, &context.config.git)
        }
        Configured::FeatureStart => change_delivery::feature_start(context, &args[0]),
        Configured::FeatureMerge => change_delivery::feature_merge(context),
        Configured::FeaturePush => change_delivery::feature_push(context),
        Configured::StateReset => change_delivery::reset_state(context),
        Configured::Approve => change_delivery::approve(context),
        Configured::ProposePin => change_delivery::propose_pin(context, &args[0]),
        Configured::Consent => change_delivery::give_consent(context),
        Configured::RevokeConsent => change_delivery::revoke_consent(context),
        Configured::ConsentedPins => change_delivery::consented_pins(context),
        Configured::MemoryCheck => project_memory::check(context),
        Configured::Resume => {
            project_memory::resume(context)?;
            Ok(0)
        }
        Configured::Commands => {
            command_runner::list_tools(context)?;
            Ok(0)
        }
        Configured::Run => run_command(context, args),
        Configured::Report => {
            command_runner::report(context)?;
            project_memory::report_checks(context)?;
            Ok(0)
        }
    }
}

fn run_command(context: &Context, args: &[String]) -> Result<i32> {
    let (name, extra) = args
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("run COMMAND [-- ARGS]"))?;
    let extra = forwarded(extra);
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
            skill: &context.caller.skill(&check.skill),
            rerun: &rerun,
        });
    }
    Ok(code)
}
