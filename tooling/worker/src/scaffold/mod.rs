mod commands;
pub mod config;
mod gate;
mod git;
mod memory;
mod package;
mod process;

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
            | "doctor"
    )
}
pub fn run(root: &Path, command: &str, args: &[String]) -> Result<i32> {
    if command == "init" {
        return package::init(root, args);
    }
    if command == "check" {
        if args.iter().any(|arg| arg != "--staged") {
            bail!("check [--staged]");
        }
        return gate::run(root, args.iter().any(|arg| arg == "--staged"));
    }
    if matches!(command, "doctor" | "memory-check" | "resume" | "report") && !args.is_empty() {
        bail!("{command} takes no arguments");
    }
    let context = config::Context::load(root).map_err(|error| {
        anyhow::anyhow!(
            "{error:#}. ACTION: Correct worker.toml and its referenced configuration/skills"
        )
    })?;
    match command {
        "doctor" => package::doctor(&context),
        "config-check" => {
            if !args.is_empty() {
                bail!("config-check takes no arguments");
            }
            let code =
                crate::lint::run(root, &context.path(&context.config.paths.lint)?, true, true)?;
            if code == 0 {
                println!(
                    "Scaffold configuration is valid (runtime {})",
                    config::VERSION
                );
            }
            Ok(code)
        }
        "guard-commit" => crate::hooks::git::guard_commit_with(
            root,
            &context.config.git.base,
            &context.config.git.prefix,
        ),
        "guard-reference" => crate::hooks::git::guard_reference_with(
            root,
            args.first().map(String::as_str).unwrap_or(""),
            &context.config.git.base,
            &context.config.git.prefix,
        ),
        "feature-start" => {
            if args.len() != 1 {
                bail!("feature-start NAME");
            }
            git::start(&context, &args[0])
        }
        "feature-merge" => {
            if !args.is_empty() {
                bail!("feature-merge takes no arguments");
            }
            git::merge(&context)
        }
        "memory-check" => memory::check(&context),
        "resume" => {
            memory::resume(&context)?;
            Ok(0)
        }
        "commands" => {
            if !args.is_empty() {
                bail!("commands takes no arguments");
            }
            for (name, spec) in &context.config.commands {
                println!("{name}: {}", spec.argv.join(" "));
            }
            Ok(0)
        }
        "run" => {
            let (name, extra) = args
                .split_first()
                .ok_or_else(|| anyhow::anyhow!("run COMMAND [-- ARGS]"))?;
            let extra = if extra.first().is_some_and(|s| s == "--") {
                &extra[1..]
            } else {
                extra
            };
            commands::run(&context, name, extra)
        }
        "report" => {
            commands::report(&context)?;
            Ok(0)
        }
        _ => bail!("unknown scaffold command {command}"),
    }
}

// The hook and executor validate the same command ID and argument contract.
pub fn hook_commands(context: &config::Context, argv: Vec<String>) -> Result<Vec<String>> {
    match argv.get(1).map(String::as_str).unwrap_or("list") {
        "run" => {
            let name = argv
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("just run requires a catalog command"))?;
            let extra = &argv[3..];
            let extra = if extra.first().is_some_and(|s| s == "--") {
                &extra[1..]
            } else {
                extra
            };
            commands::argv(context, name, extra)?;
        }
        "list" | "--list" | "resume" | "config-check" | "doctor" | "lint-rules" | "report"
        | "feature-merge" => {
            anyhow::ensure!(
                argv.len() <= 2,
                "one Just recipe is required; this recipe takes no arguments"
            );
        }
        "check" => {
            anyhow::ensure!(
                argv[2..].iter().all(|arg| arg == "--staged" || arg == "--"),
                "check [--staged]"
            );
        }
        "feature-start" => {
            let extra = argv[2..]
                .strip_prefix(&["--".to_owned()])
                .unwrap_or(&argv[2..]);
            anyhow::ensure!(extra.len() == 1, "feature-start NAME");
        }
        "lint" | "lint-config-check" => {}
        name => {
            let extra = &argv[2..];
            let extra = extra.strip_prefix(&["--".to_owned()]).unwrap_or(extra);
            commands::argv(context, name, extra)?;
        }
    }
    Ok(argv)
}
