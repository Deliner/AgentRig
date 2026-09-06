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
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct Options<'a> {
    staged: bool,
    only: Option<&'a str>,
    revision: Option<&'a str>,
}

pub fn arguments(args: &[String]) -> Result<Options<'_>> {
    let mut options = Options::default();
    let mut values = args.iter();
    while let Some(value) = values.next() {
        match value.as_str() {
            "--staged" => {
                ensure!(!options.staged, "duplicate --staged");
                options.staged = true;
            }
            "--only" | "--revision" => {
                let target = match value.as_str() {
                    "--only" => &mut options.only,
                    _ => &mut options.revision,
                };
                ensure!(target.is_none(), "duplicate {value}");
                *target = Some(
                    values
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("{value} needs a value"))?
                        .as_str(),
                );
            }
            _ => anyhow::bail!("check [--staged | --revision REV] [--only CHECK_ID]"),
        }
    }
    ensure!(
        !options.staged || options.revision.is_none(),
        "--staged and --revision cannot be combined"
    );
    Ok(options)
}
pub fn run(root: &Path, staged: bool) -> Result<i32> {
    selected(
        root,
        Options {
            staged,
            ..Options::default()
        },
    )
}
pub fn selected(root: &Path, options: Options<'_>) -> Result<i32> {
    let (snapshot, input) = prepare(root, &options)?;
    let only = options.only;
    let tree = snapshot.as_ref().map(|dir| dir.path()).unwrap_or(root);
    let context = Context::load(tree)?;
    ensure!(
        only.is_none_or(|id| context.config.checks.iter().any(|check| check.id == id)),
        "unknown check {}",
        only.unwrap_or_default()
    );
    let mut attempt = super::evidence::Attempt::start(&context, root, input, only)?;
    let code = execute_checks(&context, root, &mut attempt, only)?;
    let stable = attempt.finish(&context, root, code)?;
    ensure!(
        stable || options.revision.is_none(),
        "checked revision inputs changed; repair the check so it preserves its source inputs"
    );
    Ok(code)
}

fn prepare(
    root: &Path,
    options: &Options<'_>,
) -> Result<(Option<tempfile::TempDir>, super::evidence::Input)> {
    use super::evidence::{Input, index};
    if options.staged {
        let index = index(root)?;
        return Ok((Some(export(root)?), Input::Index(index)));
    }
    if let Some(reference) = options.revision {
        let repository = review_runner::vcs::Repository::discover(root)?
            .ok_or_else(|| anyhow::anyhow!("revision checking requires a VCS repository"))?;
        let revision = repository.resolve(reference)?;
        let snapshot = repository.export_revision(&revision)?;
        return Ok((Some(snapshot), Input::Revision(revision)));
    }
    Ok((None, Input::Worktree))
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
        let (code, broken) = checked(context, (root, attempt.exported_revision()), check, &rerun);
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

fn run_check(
    context: &Context,
    source: (&Path, Option<&str>),
    check: &Check,
    rerun: &str,
) -> Result<i32> {
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
        CheckKind::Memory => super::memory::check_with_history(context, source.0, source.1),
    }
}
fn checked(
    context: &Context,
    source: (&Path, Option<&str>),
    check: &Check,
    rerun: &str,
) -> (i32, bool) {
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
    let repository = review_runner::vcs::Repository::discover(root)?
        .ok_or_else(|| anyhow::anyhow!("index export requires a VCS repository"))?;
    let directory = repository.export_index()?;
    ensure!(
        directory.path().join(config::FILE).is_file(),
        "stage agentrig.yaml before checking the index"
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
