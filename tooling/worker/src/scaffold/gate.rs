// DECISION: D016
// DECISION: D015
// DECISION: D005
// DECISION: D004
use super::{
    commands,
    config::{self, Check, CheckKind, Context},
};
use crate::lint::{self, config::globs, inventory};
use anyhow::{Result, ensure};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn arguments(args: &[String]) -> Result<(bool, Option<&str>)> {
    let mut staged = false;
    let mut only = None;
    let mut values = args.iter();
    while let Some(value) = values.next() {
        match value.as_str() {
            "--staged" => {
                ensure!(!staged, "duplicate --staged");
                staged = true;
            }
            "--only" => {
                ensure!(only.is_none(), "duplicate --only");
                only = Some(
                    values
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("--only needs a check ID"))?
                        .as_str(),
                );
            }
            _ => anyhow::bail!("check [--staged] [--only CHECK_ID]"),
        }
    }
    Ok((staged, only))
}
pub fn run(root: &Path, staged: bool) -> Result<i32> {
    selected(root, staged, None)
}
pub fn selected(root: &Path, staged: bool, only: Option<&str>) -> Result<i32> {
    let index = staged.then(|| super::evidence::index(root)).transpose()?;
    let snapshot = if staged { Some(export(root)?) } else { None };
    let tree = snapshot.as_ref().map(|dir| dir.path()).unwrap_or(root);
    let context = Context::load(tree)?;
    ensure!(
        only.is_none_or(|id| context.config.checks.iter().any(|check| check.id == id)),
        "unknown check {}",
        only.unwrap_or_default()
    );
    let mut attempt = super::evidence::Attempt::start(&context, root, index, only)?;
    let code = execute_checks(&context, root, &mut attempt, only)?;
    attempt.finish(&context, root, code)?;
    Ok(code)
}
fn execute_checks(
    context: &Context,
    root: &Path,
    attempt: &mut super::evidence::Attempt,
    only: Option<&str>,
) -> Result<i32> {
    let files = files(context)?;
    for check in context
        .config
        .checks
        .iter()
        .filter(|check| only.is_none_or(|id| check.id == id))
    {
        let include = globs(&check.include)?;
        let applies = files.iter().any(|path| include.is_match(path));
        let skipped = !applies;
        if skipped {
            println!("SKIP [{}]: no selected files", check.id);
            continue;
        }
        let rerun = attempt.rerun(root, &check.id);
        let (code, broken) = checked(context, root, check, &rerun);
        attempt.checked(check, code, rerun)?;
        let passed = code == 0;
        if passed {
            println!("PASS [{}]", check.id);
            continue;
        }
        let stop = broken || code >= 128 || !check.warning;
        if stop {
            return Ok(code);
        }
    }
    Ok(0)
}

fn run_check(context: &Context, root: &Path, check: &Check, rerun: &str) -> Result<i32> {
    match check.kind {
        CheckKind::Command => commands::run(
            context,
            check.command.as_deref().expect("validated command"),
            &[],
        ),
        CheckKind::Lint => lint::check(
            &context.root,
            &context.path(&context.config.paths.lint)?,
            rerun,
        ),
        CheckKind::Memory => super::memory::check_with_history(context, root),
    }
}
fn checked(context: &Context, root: &Path, check: &Check, rerun: &str) -> (i32, bool) {
    let result = run_check(context, root, check, rerun);
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
        eprintln!(
            "{}",
            crate::diagnostics::Guidance {
                level,
                id: &check.id,
                location: &check.include.join(", "),
                message: &message,
                skill: &check.skill,
                rerun
            }
        );
    }
    (code, broken)
}
fn export(root: &Path) -> Result<tempfile::TempDir> {
    let directory = tempfile::tempdir()?;
    let prefix = format!("--prefix={}/", directory.path().display());
    let result = Command::new("git")
        .args(["checkout-index", "--all", &prefix])
        .current_dir(root)
        .output()?;
    ensure!(
        result.status.success(),
        "cannot export index: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    ensure!(
        directory.path().join(config::FILE).is_file(),
        "stage worker.toml before checking the index"
    );
    Ok(directory)
}
pub fn files(context: &Context) -> Result<Vec<PathBuf>> {
    let excludes = vec![
        ".git".into(),
        ".git/**".into(),
        context.config.paths.runtime.clone(),
        format!("{}/**", context.config.paths.runtime),
    ];
    Ok(inventory::collect(&context.root, &globs(&excludes)?)?.files)
}
