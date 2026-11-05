use crate::project_memory::evidence::{Input, native_index};
use crate::shared::console;
mod selection;
// DECISION: D016
// DECISION: D015
// DECISION: D005
// DECISION: D004
use crate::shared::paths::patterns::globs;
use crate::structural_lint::{self, inventory};
use crate::{
    command_runner,
    project_config::{self, Check, CheckKind, Context},
};
use anyhow::{Result, ensure};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct Options<'a> {
    staged: bool,
    only: Option<&'a str>,
    revision: Option<&'a str>,
    base: Option<&'a str>,
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
pub fn branch(root: &Path, base: &str) -> Result<i32> {
    selected(
        root,
        Options {
            base: Some(base),
            ..Options::default()
        },
    )
}
pub fn selected(root: &Path, options: Options<'_>) -> Result<i32> {
    let (snapshot, input) = prepare(root, &options)?;
    let only = options.only;
    let tree = snapshot.as_ref().map(|dir| dir.path()).unwrap_or(root);
    let mut context = Context::load(tree)?;
    context.runtime = project_config::relative(root, &context.config.paths.runtime)?;
    if options.staged {
        crate::shared::vcs::Repository::new(root).index_entries()?;
    }
    ensure!(
        only.is_none_or(|id| context.config.checks.iter().any(|check| check.id == id)),
        "unknown check {}",
        only.unwrap_or_default()
    );
    let mut attempt = crate::project_memory::evidence::Attempt::start(&context, root, input, only)?;
    let code = execute_checks(&context, root, &mut attempt, &options)?;
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
) -> Result<(
    Option<tempfile::TempDir>,
    crate::project_memory::evidence::Input,
)> {
    if options.staged {
        let index = native_index(root)?;
        return Ok((Some(export(root)?), Input::Index(index)));
    }
    if let Some(reference) = options.revision {
        let repository = crate::shared::vcs::Repository::discover(root)?
            .ok_or_else(|| anyhow::anyhow!("revision checking requires a VCS repository"))?;
        let revision = repository.resolve(reference)?;
        let snapshot = repository.export_revision(&revision)?;
        return Ok((Some(snapshot), Input::Revision(revision)));
    }
    Ok((None, Input::Worktree))
}
fn change_index(options: &Options<'_>) -> selection::Index {
    if options.staged {
        selection::Index::Staged
    } else {
        selection::Index::Working
    }
}
fn execute_checks(
    context: &Context,
    root: &Path,
    attempt: &mut crate::project_memory::evidence::Attempt,
    options: &Options<'_>,
) -> Result<i32> {
    let files = files(context)?;
    let index = change_index(options);
    let changes = selection::changes(root, index, options.base)?;
    for check in context
        .config
        .checks
        .iter()
        .filter(|check| options.only.is_none_or(|id| check.id == id))
    {
        if let Some(reason) = skip_reason(check, &files, options)? {
            console::stdout(format!("SKIP [{}]: {reason}", check.id));
            continue;
        }
        let targets = selection::targets(check, changes.as_deref())?;
        if let Some(targets) = &targets {
            attempt.selective()?;
            console::stdout(format!("SELECT [{}]: {targets:?}", check.id));
            let empty = targets.is_empty();
            if empty {
                continue;
            }
        }
        let source = Selected {
            root,
            targets: targets.as_deref().unwrap_or_default(),
        };
        if let Some(code) = execute_selected(context, source, check, attempt)? {
            return Ok(code);
        }
    }
    Ok(0)
}

fn skip_reason(
    check: &Check,
    files: &[PathBuf],
    options: &Options<'_>,
) -> Result<Option<&'static str>> {
    let include = globs(&check.include)?;
    let applies = files.iter().any(|path| include.is_match(path));
    let commit_of_merge_check = options.staged && check.kind == CheckKind::Tools;
    Ok(match (applies, commit_of_merge_check) {
        (false, _) => Some("no selected files"),
        (true, true) => Some("checked before merge"),
        (true, false) => None,
    })
}
struct Selected<'a> {
    root: &'a Path,
    targets: &'a [String],
}
struct CheckRun<'a> {
    root: &'a Path,
    revision: Option<&'a str>,
    targets: &'a [String],
}
struct Outcome {
    code: i32,
    broken: bool,
}
fn execute_selected(
    context: &Context,
    source: Selected<'_>,
    check: &Check,
    attempt: &mut crate::project_memory::evidence::Attempt,
) -> Result<Option<i32>> {
    let rerun = attempt.rerun(source.root, &check.id);
    let input = CheckRun {
        root: source.root,
        revision: attempt.exported_revision(),
        targets: source.targets,
    };
    let Outcome { code, broken } = checked(context, input, check, &rerun);
    attempt.checked(check, code, rerun)?;
    let passed = code == 0;
    if passed {
        console::stdout(format!("PASS [{}]", check.id));
        return Ok(None);
    }
    let stop = broken || code >= 128 || !check.warning;
    Ok(stop.then_some(code))
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
        CheckKind::Tools => command_runner::registry::check::run(context, check, rerun),
    }
}
fn checked(context: &Context, source: CheckRun<'_>, check: &Check, rerun: &str) -> Outcome {
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
fn export(root: &Path) -> Result<tempfile::TempDir> {
    let repository = crate::shared::vcs::Repository::discover(root)?
        .ok_or_else(|| anyhow::anyhow!("index export requires a VCS repository"))?;
    let directory = repository.export_index()?;
    ensure!(
        directory.path().join(project_config::FILE).is_file(),
        "stage {} before checking the index",
        project_config::FILE
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
