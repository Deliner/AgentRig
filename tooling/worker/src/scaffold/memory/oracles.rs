use super::super::{
    commands,
    config::{Context, Oracle, Runner},
};
use anyhow::{Context as _, Result, ensure};
use std::path::Path;

pub fn validate(context: &Context, id: &str, function: &str, source: &Path) -> Result<()> {
    let oracle = context
        .config
        .oracles
        .get(id)
        .with_context(|| format!("{id}: configure a test oracle"))?;
    let check = context
        .config
        .checks
        .iter()
        .find(|c| c.id == oracle.check)
        .expect("validated oracle check");
    let name = check.command.as_deref().expect("validated command check");
    let target_function = oracle.target.rsplit("::").next().unwrap_or("");
    let target_function = target_function.split('[').next().unwrap_or("");
    ensure!(
        target_function == function,
        "{id}: oracle target does not match linked function {function}"
    );
    let python = matches!(oracle.runner, Runner::Pytest);
    if python {
        let file = oracle.target.split("::").next().unwrap_or("");
        let cwd = context.path(&context.config.commands[name].cwd)?;
        ensure!(
            crate::paths::resolve(&cwd.join(file))? == source,
            "{id}: oracle target does not match linked source"
        );
    }
    discover(context, id, name, oracle)
}
fn discovery_arguments(mut argv: Vec<String>, oracle: &Oracle) -> Vec<String> {
    match oracle.runner {
        Runner::Pytest => {
            argv.extend(["--collect-only".into(), "-q".into(), oracle.target.clone()]);
        }
        Runner::Cargo => {
            let separator = argv
                .iter()
                .position(|arg| arg == "--")
                .unwrap_or(argv.len());
            argv.insert(separator, oracle.target.clone());
            let missing_separator = !argv.iter().any(|arg| arg == "--");
            if missing_separator {
                argv.push("--".into());
            }
            argv.push("--list".into());
        }
    }
    argv
}
fn discover(context: &Context, id: &str, name: &str, oracle: &Oracle) -> Result<()> {
    let argv = discovery_arguments(commands::argv(context, name, &[])?, oracle);
    let result = commands::execute(context, name, argv, true)?;
    ensure!(
        result.status.success(),
        "{id}: test discovery failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    let found = match oracle.runner {
        Runner::Pytest => stdout
            .lines()
            .any(|line| line == oracle.target || line.starts_with(&format!("{}[", oracle.target))),
        Runner::Cargo => stdout
            .lines()
            .any(|line| line == format!("{}: test", oracle.target)),
    };
    ensure!(
        found,
        "{id}: test target {} was not discovered",
        oracle.target
    );
    Ok(())
}

pub fn function_target(context: &Context, id: &str) -> Result<String> {
    let oracle = context
        .config
        .oracles
        .get(id)
        .with_context(|| format!("{id}: configure a test oracle"))?;
    let target = oracle.target.split('[').next().unwrap_or("");
    Ok(match oracle.runner {
        Runner::Cargo => target.to_owned(),
        Runner::Pytest => target
            .split_once("::")
            .map(|(_, function)| function)
            .unwrap_or("")
            .to_owned(),
    })
}
