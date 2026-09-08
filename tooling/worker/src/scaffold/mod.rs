mod commands;
pub mod config;
mod evidence;
mod gate;
mod git;
mod memory;
mod package;
mod process;
pub(crate) mod receipt;
pub(crate) mod recovery;
pub(crate) mod settings;
pub(crate) mod upgrade;

use anyhow::{Result, bail};
use std::path::Path;

pub fn owns(command: &str) -> bool {
    matches!(
        command,
        "config-check"
            | "config-inspect"
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
        "config-inspect" => return package::setup::inspect(root, args),
        "upgrade" => return upgrade::run(root, args),
        "guard-commit" => return guard_commit(root, args),
        "resume" if upgrade::recovery::configuration_pending(root)? => {
            return resume_upgrade(root, args);
        }
        "check" => {
            return gate::selected(root, gate::arguments(args)?);
        }
        _ => {}
    }
    validate_arguments(command, args)?;
    let delivery = matches!(command, "guard-commit" | "feature-merge");
    if delivery {
        upgrade::recovery::guard(root)?;
    }
    let recovery = matches!(command, "run" | "resume" | "commands");
    let context = config::Context::load_for(root, recovery)?;
    let jobs = matches!(
        command,
        "jobs" | "job-status" | "job-logs" | "job-stop" | "job-cleanup"
    );
    if jobs {
        return crate::jobs::cli(&context.path(&context.config.paths.runtime)?, command, args);
    }
    match command {
        "job-start" => commands::background(&context, args),
        "_job-run" => commands::background_run(&context, args),
        _ => configured_command(&context, command, args),
    }
}
fn resume_upgrade(root: &Path, args: &[String]) -> Result<i32> {
    validate_arguments("resume", args)?;
    println!(
        "{}",
        serde_json::json!({
            "snapshot": "unavailable",
            "upgrade": upgrade::recovery::journal(root)?,
            "guidance": upgrade::recovery::guidance(root)?,
        })
    );
    Ok(0)
}
fn guard_commit(root: &Path, args: &[String]) -> Result<i32> {
    upgrade::recovery::guard(root)?;
    let reference = match args {
        [] => None,
        [flag, revision] if flag == "--revision" => Some(revision.as_str()),
        _ => bail!("guard-commit [--revision REV]"),
    };
    let snapshot = match reference {
        Some(revision) => {
            let config = config::read(root)?;
            let repository = config
                .vcs
                .backend
                .repository_source(root)?
                .ok_or_else(|| anyhow::anyhow!("commit guard requires a VCS repository"))?;
            let revision = repository.resolve(revision)?;
            Some((repository.export_revision(&revision)?, revision))
        }
        None => None,
    };
    let tree = snapshot
        .as_ref()
        .map(|(snapshot, _)| snapshot.path())
        .unwrap_or(root);
    let context = config::Context::load(tree)?;
    let reference = snapshot.as_ref().map(|(_, revision)| revision.as_str());
    git::guard_commit_with(root, &context.config.vcs, reference)
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
    let git = &context.config.vcs;
    match command {
        "doctor" => package::doctor(context),
        "config-check" => config_check(context),
        "guard-reference" => git::guard_reference_with(
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
