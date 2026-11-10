use crate::project_config::{Check, CheckKind, Context};
use crate::shared::console;
use crate::{command_runner, structural_lint};
use anyhow::Result;
use std::path::Path;

pub(super) struct CheckRun<'a> {
    pub(super) root: &'a Path,
    pub(super) revision: Option<&'a str>,
    pub(super) targets: &'a [String],
}
pub(super) struct Outcome {
    pub(super) code: i32,
    pub(super) broken: bool,
}

fn run_check(context: &Context, source: CheckRun<'_>, check: &Check, rerun: &str) -> Result<i32> {
    match check.kind {
        CheckKind::Command => {
            let command = check
                .command
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("command check without a command"))?;
            command_runner::run(context, command, source.targets)
        }
        CheckKind::Lint => structural_lint::check(
            &context.root,
            &context.path(&context.config.paths.lint)?,
            rerun,
        ),
        CheckKind::Memory => {
            crate::project_memory::check_with_history(context, source.root, source.revision)
        }
        CheckKind::Tools => {
            let tree = command_runner::registry::check::Source {
                repository: source.root,
                exported_revision: source.revision,
            };
            command_runner::registry::check::run(context, tree, check, rerun)
        }
    }
}
pub(super) fn checked(
    context: &Context,
    source: CheckRun<'_>,
    check: &Check,
    rerun: &str,
) -> Outcome {
    let result = run_check(context, source, check, rerun);
    let (code, message, broken) = match result {
        Ok(code) => (
            code,
            format!("exited {code}; see original output above"),
            false,
        ),
        Err(error) => (2, format!("{error:#}"), true),
    };
    let failed = code != 0;
    if failed {
        let advisory = check.warning && !broken;
        let level = if advisory { "WARNING" } else { "ERROR" };
        console::stderr(crate::shared::diagnostics::Guidance {
            level,
            id: &check.id,
            location: &check.include.join(", "),
            message: &message,
            skill: &check.skill,
            rerun,
        });
    }
    Outcome { code, broken }
}
