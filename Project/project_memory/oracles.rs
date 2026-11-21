use super::source::{Invariant, Source};
use crate::{
    command_runner,
    project_config::{Context, Oracle, Runner},
};
use anyhow::{Context as _, Result, ensure};
use std::path::Path;

pub struct Discovery<'a> {
    id: &'a str,
    command: &'a str,
    oracle: &'a Oracle,
}

pub fn validate<'a>(
    context: &'a Context,
    invariant: Invariant<'_>,
    source: &Path,
) -> Result<Discovery<'a>> {
    let Invariant { id, function } = invariant;
    let (configured, oracle) = context
        .config
        .oracles
        .get_key_value(id)
        .with_context(|| format!("{id}: configure a test oracle"))?;
    let command = command(context, oracle)?;
    let target_function = oracle.target.rsplit("::").next().unwrap_or("");
    let target_function = target_function.split('[').next().unwrap_or("");
    ensure!(
        target_function == function,
        "{id}: oracle target does not match linked function {function}"
    );
    match oracle.runner {
        Runner::Pytest => {
            let file = oracle.target.split("::").next().unwrap_or("");
            let cwd = context.path(&context.config.commands[command].cwd)?;
            ensure!(
                crate::shared::paths::resolve(&cwd.join(file))? == source,
                "{id}: oracle target does not match linked source"
            );
        }
        Runner::Cargo => {}
    }
    Ok(Discovery {
        id: configured,
        command,
        oracle,
    })
}
fn command<'a>(context: &'a Context, oracle: &Oracle) -> Result<&'a str> {
    let check = context
        .config
        .checks
        .iter()
        .find(|c| c.id == oracle.check)
        .context("oracle check is not configured")?;
    check
        .command
        .as_deref()
        .context("oracle check has no command")
}
pub fn discover(context: &Context, discoveries: &[Discovery<'_>]) -> Result<()> {
    let mut pending: Vec<&Discovery<'_>> = discoveries.iter().collect();
    while let Some(&first) = pending.first() {
        let (group, rest): (Vec<&Discovery<'_>>, Vec<&Discovery<'_>>) =
            pending.into_iter().partition(|discovery| {
                discovery.command == first.command && discovery.oracle.runner == first.oracle.runner
            });
        discover_group(context, &group)?;
        pending = rest;
    }
    Ok(())
}
fn discovery_arguments(mut argv: Vec<String>, runner: &Runner, targets: &[&str]) -> Vec<String> {
    match runner {
        Runner::Pytest => {
            argv.extend(["--collect-only".into(), "-q".into()]);
            argv.extend(targets.iter().map(|target| (*target).to_owned()));
        }
        Runner::Cargo => {
            let missing_separator = !argv.iter().any(|arg| arg == "--");
            if missing_separator {
                argv.push("--".into());
            }
            argv.push("--list".into());
        }
    }
    argv
}
fn discover_group(context: &Context, group: &[&Discovery<'_>]) -> Result<()> {
    let Some(first) = group.first() else {
        return Ok(());
    };
    let runner = &first.oracle.runner;
    let targets: Vec<&str> = group
        .iter()
        .map(|each| each.oracle.target.as_str())
        .collect();
    let argv = command_runner::argv(context, first.command, &[])?;
    let argv = discovery_arguments(argv, runner, &targets);
    let capture = command_runner::Capture::Collect;
    let result = command_runner::execute(context, first.command, argv, capture)?;
    let ids: Vec<&str> = group.iter().map(|each| each.id).collect();
    ensure!(
        result.status.success(),
        "{}: test discovery failed: {}",
        ids.join(", "),
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    for discovery in group {
        let target = &discovery.oracle.target;
        let found = match runner {
            Runner::Pytest => stdout
                .lines()
                .any(|line| line == target || line.starts_with(&format!("{target}["))),
            Runner::Cargo => stdout.lines().any(|line| line == format!("{target}: test")),
        };
        ensure!(
            found,
            "{}: test target {target} was not discovered",
            discovery.id
        );
    }
    Ok(())
}

pub fn marked(context: &Context, id: &str, source: &Source) -> Result<bool> {
    let oracle = context
        .config
        .oracles
        .get(id)
        .with_context(|| format!("{id}: configure a test oracle"))?;
    let target = oracle.target.split('[').next().unwrap_or("");
    Ok(match oracle.runner {
        // DECISION: D157
        Runner::Cargo => source.marked_function(id, |name| {
            target
                .strip_suffix(name)
                .is_some_and(|module| module.is_empty() || module.ends_with("::"))
        }),
        Runner::Pytest => {
            let function = target.split_once("::").map_or("", |(_, function)| function);
            source.marked_function(id, |name| name == function)
        }
    })
}
