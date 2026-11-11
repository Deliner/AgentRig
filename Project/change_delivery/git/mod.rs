// DECISION: D010
// DECISION: D019
// DECISION: D035
mod integration;
use super::gate;
use crate::project_config::Context;
use crate::shared::console;
use crate::shared::vcs::{Revisions, git_context as git};
use anyhow::{Result, ensure};
use std::{
    io::{self, Read},
    path::Path,
};

pub fn start(context: &Context, name: &str) -> Result<i32> {
    let settings = &context.config.git;
    let repository = crate::shared::vcs::Repository::discover(&context.root)?
        .ok_or_else(|| anyhow::anyhow!("feature-start requires a VCS repository"))?;
    let branch = integration::start(&repository, settings, name)?;
    console::stdout(branch);
    Ok(0)
}

pub fn merge(context: &Context) -> Result<i32> {
    let settings = &context.config.git;
    let repository = crate::shared::vcs::Repository::discover(&context.root)?
        .ok_or_else(|| anyhow::anyhow!("integration requires a VCS repository"))?;
    let (feature, code) = integration::integrate(
        &repository,
        settings,
        || checked(context),
        |args| execute_vcs(&repository, args),
    )?;
    let passed = code == 0;
    if passed {
        console::stdout(format!("merged {feature}; branch retained"));
        return cleanup_merged(context, &feature);
    }
    Ok(code)
}

fn checked(context: &Context) -> Result<i32> {
    let code = gate::branch(&context.root, &context.config.git.base)?;
    let passed = code == 0;
    if passed {
        ensure!(
            crate::project_memory::evidence::resume(context)["current"] == true,
            "checked merge inputs changed; preserve the native operation and retry after repairing the checks"
        );
    }
    Ok(code)
}

fn execute_vcs(repository: &crate::shared::vcs::Repository<'_>, args: &[&str]) -> Result<i32> {
    let output = crate::command_runner::jobs::process::execute(
        &mut repository.command(args),
        crate::command_runner::jobs::Capture::Stream,
        None,
    )?;
    Ok(crate::command_runner::jobs::process::exit_code(&output))
}

fn cleanup_merged(context: &Context, feature: &str) -> Result<i32> {
    let scratch = crate::command_runner::scratch::cleanup(context, feature);
    if let Err(error) = &scratch {
        console::stderr(format!(
            "merge succeeded; scratch area cleanup failed: {error:#}"
        ));
    }
    Ok(cleanup_jobs(context, feature)?.max(i32::from(scratch.is_err())))
}
fn cleanup_jobs(context: &Context, feature: &str) -> Result<i32> {
    let identified = crate::command_runner::jobs::owner().is_some();
    let anonymous = !identified;
    if anonymous {
        return Ok(0);
    }
    let result = crate::command_runner::jobs::cleanup::run(
        &context.path(&context.config.paths.runtime)?,
        Some(feature),
    );
    match result {
        Ok(report) => {
            let failed = report["errors"]
                .as_array()
                .is_some_and(|errors| !errors.is_empty());
            console::stdout(format!("post-merge cleanup: {report}"));
            if failed {
                console::stderr(format!(
                    "merge succeeded; cleanup failed. Retry job-cleanup --branch {feature}"
                ));
            }
            Ok(i32::from(failed))
        }
        Err(error) => {
            console::stderr(format!(
                "merge succeeded; cleanup failed: {error}. Retry job-cleanup --branch {feature}"
            ));
            Ok(1)
        }
    }
}

// DECISION: D015
pub fn guard_commit(root: &Path, settings: &crate::shared::vcs::Settings) -> Result<i32> {
    let repository = crate::shared::vcs::Repository::discover(root)?
        .ok_or_else(|| anyhow::anyhow!("commit guard requires a VCS repository"))?;
    let (branch, merge) = repository.commit_context()?;
    let base = settings.base.as_str();
    let prefix = settings.prefix.as_str();
    let feature = branch.starts_with(prefix);
    let base_merge = branch == base && merge;
    let permitted = feature || base_merge;
    if permitted {
        return Ok(0);
    }
    console::stderr(format!(
        "direct commits on {base} are prohibited; create a {prefix} branch"
    ));
    Ok(1)
}
pub fn guard_reference_with(
    root: &Path,
    phase: &str,
    settings: &crate::shared::vcs::Settings,
) -> Result<i32> {
    let no_validation_needed = phase != "prepared";
    if no_validation_needed {
        return Ok(0);
    }
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    for line in input.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let malformed = fields.len() != 3;
        if malformed {
            return Ok(1);
        }
        if let [old, new, reference] = fields.as_slice() {
            let deletion = new.chars().all(|ch| ch == '0')
                && reference.starts_with(&format!("refs/heads/{}", settings.prefix));
            if deletion {
                let merged = Deletion { old, reference }.merged(root, &settings.base)?;
                if merged {
                    console::stderr("merged feature branches must be retained");
                    return Ok(1);
                }
            }
        }
    }
    Ok(0)
}

struct Deletion<'a> {
    old: &'a str,
    reference: &'a str,
}

impl Deletion<'_> {
    fn merged(self, root: &Path, base: &str) -> Result<bool> {
        let unknown_tip = self.old.chars().all(|ch| ch == '0');
        let tip = if unknown_tip {
            let commit = format!("{reference}^{{commit}}", reference = self.reference);
            git(root, &["rev-parse", "--verify", &commit])?
        } else {
            self.old.to_owned()
        };
        let revisions = Revisions {
            from: &tip,
            to: base,
        };
        crate::shared::vcs::Repository::new(root).ancestor(revisions)
    }
}
