mod commands;
pub mod config;
mod evidence;
mod gate;
mod git;
mod memory;
mod package;
mod process;
pub(crate) mod upgrade;

use anyhow::{Result, bail};
use std::path::Path;

pub fn owns(command: &str) -> bool {
    matches!(
        command,
        "config-check"
            | "run"
            | "commands"
            | "report"
            | "check"
            | "memory-check"
            | "resume"
            | "feature-start"
            | "feature-merge"
            | "init"
            | "setup"
            | "doctor"
            | "upgrade"
            | "jobs"
            | "job-status"
            | "job-logs"
            | "job-start"
            | "job-stop"
            | "job-cleanup"
            | "_job-run"
    )
}
pub fn run(root: &Path, command: &str, args: &[String]) -> Result<i32> {
    match command {
        "init" => return package::init(root, args),
        "setup" => return package::setup(root, args),
        "upgrade" => return upgrade::run(root, args),
        "check" => {
            let (staged, only) = gate::arguments(args)?;
            return gate::selected(root, staged, only);
        }
        _ => {}
    }
    validate_arguments(command, args)?;
    let delivery = matches!(command, "guard-commit" | "feature-merge");
    if delivery {
        upgrade::recovery::guard(root)?;
    }
    let recovery = matches!(command, "run" | "resume" | "commands");
    let context = config::Context::load_for(root, recovery).map_err(|error| {
        anyhow::anyhow!(
            "{error:#}. ACTION: Correct worker.toml and its referenced configuration/skills"
        )
    })?;
    let jobs = matches!(
        command,
        "jobs" | "job-status" | "job-logs" | "job-stop" | "job-cleanup"
    );
    if jobs {
        return discipline_worker::jobs::cli(
            &context.path(&context.config.paths.runtime)?,
            command,
            args,
        );
    }
    match command {
        "job-start" => commands::background(&context, args),
        "_job-run" => commands::background_run(&context, args),
        _ => configured_command(&context, command, args),
    }
}
fn validate_arguments(command: &str, args: &[String]) -> Result<()> {
    let unexpected = matches!(
        command,
        "doctor"
            | "memory-check"
            | "resume"
            | "report"
            | "config-check"
            | "commands"
            | "feature-merge"
    ) && !args.is_empty();
    if unexpected {
        bail!("{command} takes no arguments");
    }
    let invalid_start = command == "feature-start" && args.len() != 1;
    if invalid_start {
        bail!("feature-start NAME");
    }
    Ok(())
}
fn configured_command(context: &config::Context, command: &str, args: &[String]) -> Result<i32> {
    let root = &context.root;
    let git = &context.config.git;
    match command {
        "doctor" => package::doctor(context),
        "config-check" => config_check(context),
        "guard-commit" => crate::hooks::git::guard_commit_with(root, &git.base, &git.prefix),
        "guard-reference" => crate::hooks::git::guard_reference_with(
            root,
            args.first().map(String::as_str).unwrap_or(""),
            &git.base,
            &git.prefix,
        ),
        "feature-start" => git::start(context, &args[0]),
        "feature-merge" => git::merge(context),
        "memory-check" => memory::check(context),
        "resume" => {
            memory::resume(context)?;
            Ok(0)
        }
        "commands" => {
            for (name, spec) in &context.config.commands {
                println!("{name}: {}", spec.argv.join(" "));
            }
            Ok(0)
        }
        "run" => {
            let (name, extra) = args
                .split_first()
                .ok_or_else(|| anyhow::anyhow!("run COMMAND [-- ARGS]"))?;
            run_command(context, name, forwarded(extra))
        }
        "report" => {
            commands::report(context)?;
            evidence::report(context)?;
            Ok(0)
        }
        _ => bail!("unknown scaffold command {command}"),
    }
}
fn run_command(context: &config::Context, name: &str, extra: &[String]) -> Result<i32> {
    let code = commands::run(context, name, extra)?;
    let failed = code != 0;
    let check = context
        .config
        .checks
        .iter()
        .find(|check| check.command.as_deref() == Some(name));
    if let Some(check) = check.filter(|_| failed) {
        let mut args = vec!["run".into(), name.into(), "--".into()];
        args.extend_from_slice(extra);
        let rerun = crate::diagnostics::rerun(&context.root, &args);
        eprintln!(
            "{}",
            crate::diagnostics::Guidance {
                level: "ERROR",
                id: &check.id,
                location: &context.config.commands[name].cwd,
                message: &format!("exited {code}; see original output above"),
                skill: &check.skill,
                rerun: &rerun
            }
        );
    }
    Ok(code)
}
fn config_check(context: &config::Context) -> Result<i32> {
    let code = if context.config.capabilities.lint {
        crate::lint::run(
            &context.root,
            &context.path(&context.config.paths.lint)?,
            true,
            true,
        )?
    } else {
        0
    };
    let valid = code == 0;
    if valid {
        println!(
            "Scaffold configuration is valid (runtime {})",
            config::VERSION
        );
    }
    Ok(code)
}
fn forwarded(args: &[String]) -> &[String] {
    args.strip_prefix(&["--".to_owned()]).unwrap_or(args)
}

// The hook and executor validate the same command ID and argument contract.
pub fn hook_commands(context: &config::Context, argv: Vec<String>) -> Result<Vec<String>> {
    match argv.get(1).map(String::as_str).unwrap_or("list") {
        "run" => {
            let name = argv
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("just run requires a catalog command"))?;
            commands::argv(context, name, forwarded(&argv[3..]))?;
        }
        "list" | "--list" | "resume" | "config-check" | "doctor" | "lint-rules" | "report"
        | "feature-merge" => {
            anyhow::ensure!(
                argv.len() <= 2,
                "one Just recipe is required; this recipe takes no arguments"
            );
        }
        "upgrade" => upgrade::arguments(forwarded(&argv[2..]))?,
        "check" => {
            gate::arguments(forwarded(&argv[2..]))?;
        }
        "feature-start" => {
            let extra = forwarded(&argv[2..]);
            anyhow::ensure!(extra.len() == 1, "feature-start NAME");
        }
        "lint" | "lint-config-check" | "lint-rule" | "lint-explain" | "review" | "setup"
        | "jobs" | "job-status" | "job-logs" | "job-stop" | "job-start" | "job-cleanup"
        | "delegate" => {}
        name => {
            commands::argv(context, name, forwarded(&argv[2..]))?;
        }
    }
    Ok(argv)
}
