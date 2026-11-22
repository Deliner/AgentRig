// DECISION: D233
use super::{approval, compaction, consent, proposal};
use crate::command_runner::{finished_runs, jobs_directory};
use crate::project_config::Context;
use crate::shared::console;
use crate::shared::vcs::{Removal, Repository, remove_ignored};
use anyhow::Result;
use globset::GlobSet;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub(super) fn ignored(context: &Context, repository: &Repository<'_>) -> i32 {
    let planned = Plan::of(context, repository);
    let plan = match planned {
        Ok(plan) => plan,
        Err(error) => {
            console::stderr(format!(
                "merge succeeded; ignored files were kept: {error:#}; the next merge retries"
            ));
            return 1;
        }
    };
    let kept = |path: &Path| plan.kept.holds(path);
    let runs = remove_ignored(&plan.runs, &kept);
    let entries = remove_ignored(&plan.entries, &kept);
    let mut lines = Vec::new();
    let count = runs.removed.len();
    let pruned = count > 0;
    if pruned {
        let jobs = plan.kept.relative(&plan.jobs);
        let bytes: u64 = runs.removed.iter().map(|removed| removed.bytes).sum();
        lines.push(Line {
            name: format!("{jobs}: {count} finished runs"),
            bytes,
        });
    }
    lines.extend(entries.removed.iter().map(|removed| Line {
        name: plan.kept.relative(&removed.path),
        bytes: removed.bytes,
    }));
    report(&lines, [runs, entries])
}

struct Plan {
    kept: Kept,
    jobs: PathBuf,
    runs: Vec<PathBuf>,
    entries: Vec<PathBuf>,
}

impl Plan {
    fn of(context: &Context, repository: &Repository<'_>) -> Result<Self> {
        let runtime = context.path(&context.config.paths.runtime)?;
        let jobs = jobs_directory(&runtime);
        let paths = vec![
            approval::record_path(context)?,
            consent::location(context)?,
            proposal::mirror(context)?,
            compaction::location(context)?,
            jobs.clone(),
        ];
        let kept = Kept {
            root: context.root.clone(),
            paths,
            revision: pinned(context, repository)?,
            patterns: context.config.keep_matcher()?,
        };
        let entries: Vec<PathBuf> = repository
            .ignored()?
            .iter()
            .map(|entry| context.root.join(entry.trim_suffix('/')))
            .collect();
        Ok(Self {
            kept,
            runs: finished_runs(&runtime)?,
            jobs,
            entries,
        })
    }
}

fn pinned(context: &Context, repository: &Repository<'_>) -> Result<Option<String>> {
    let Some(pin) = context.protection.pin() else {
        return Ok(None);
    };
    let committed = String::from_utf8(repository.read("HEAD", pin)?)?;
    Ok(Some(committed.trim().to_owned()))
}

struct Kept {
    root: PathBuf,
    paths: Vec<PathBuf>,
    revision: Option<String>,
    patterns: GlobSet,
}

impl Kept {
    fn holds(&self, path: &Path) -> bool {
        let installed = |revision: &String| path.file_name() == Some(OsStr::new(revision));
        self.paths.iter().any(|kept| kept == path)
            || self.revision.as_ref().is_some_and(installed)
            || self.patterns.is_match(path.trim_prefix(&self.root))
    }

    fn relative(&self, path: &Path) -> String {
        path.trim_prefix(&self.root).display().to_string()
    }
}

struct Line {
    name: String,
    bytes: u64,
}

fn report(lines: &[Line], removals: [Removal; 2]) -> i32 {
    let any = !lines.is_empty();
    if any {
        let freed = size(lines.iter().map(|line| line.bytes).sum());
        let listed: Vec<String> = lines
            .iter()
            .map(|line| {
                let name = &line.name;
                let bytes = size(line.bytes);
                format!("  {name} ({bytes})")
            })
            .collect();
        let listed = listed.join("\n");
        console::stdout(format!("removed ignored files, freeing {freed}:\n{listed}"));
    }
    let failures: Vec<String> = removals
        .into_iter()
        .flat_map(|removal| removal.failures)
        .collect();
    for failure in &failures {
        console::stderr(format!(
            "merge succeeded; could not {failure}; the next merge retries"
        ));
    }
    i32::from(!failures.is_empty())
}

fn size(bytes: u64) -> String {
    const LARGER: [&str; 4] = ["KiB", "MiB", "GiB", "TiB"];
    let mut scale = 1_u64;
    let mut unit = "B";
    for larger in LARGER {
        let below = bytes < scale * 1024;
        if below {
            break;
        }
        scale *= 1024;
        unit = larger;
    }
    let whole_bytes = scale == 1;
    if whole_bytes {
        return format!("{bytes} B");
    }
    let tenths = u128::from(bytes) * 10 / u128::from(scale);
    let whole = tenths / 10;
    let fraction = tenths % 10;
    format!("{whole}.{fraction} {unit}")
}
