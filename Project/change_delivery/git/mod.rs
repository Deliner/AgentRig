// DECISION: D010
// DECISION: D019
// DECISION: D035
pub mod approval;
mod integration;
pub mod pin;
mod push;
mod state;
#[cfg(test)]
mod tests;
use super::gate;
use crate::command_runner::sandbox::{CATALOG_ROOT, catalog_run};
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
        context,
        || checked(context),
        |args| execute_vcs(&repository, args),
    )?;
    let passed = code == 0;
    if passed {
        console::stdout(format!("merged {feature}; branch retained"));
        let pushed = push::merged(&repository, settings, &feature);
        return Ok(cleanup_jobs(context, &feature)?.max(pushed));
    }
    Ok(code)
}

// DECISION: D191
pub fn reset_state(context: &Context) -> Result<i32> {
    let repository = crate::shared::vcs::Repository::discover(&context.root)?
        .ok_or_else(|| anyhow::anyhow!("state-reset requires a VCS repository"))?;
    let (branch, _) = repository.commit_context()?;
    let feature = branch.starts_with(&context.config.git.prefix);
    if feature {
        state::State::of(context)?.reset(&repository)?;
    }
    Ok(0)
}

pub fn push(context: &Context) -> Result<i32> {
    let repository = crate::shared::vcs::Repository::discover(&context.root)?
        .ok_or_else(|| anyhow::anyhow!("feature-push requires a VCS repository"))?;
    push::committed(&repository, &context.config.git)
}

fn checked(context: &Context) -> Result<i32> {
    let code = gate::branch(&context.root, &context.config.git.base)?;
    let passed = code == 0;
    if passed {
        ensure!(
            crate::project_memory::check_evidence::resume(context)["current"] == true,
            "checked merge inputs changed; preserve the native operation and retry after repairing the checks"
        );
    }
    Ok(code)
}

fn execute_vcs(repository: &crate::shared::vcs::Repository<'_>, args: &[&str]) -> Result<i32> {
    let mut command = repository.command(args);
    // DECISION: D204
    command.env_remove(CATALOG_ROOT);
    let output = crate::command_runner::jobs::process::execute(
        &mut command,
        crate::command_runner::jobs::Capture::Stream,
        None,
    )?;
    Ok(crate::command_runner::jobs::process::exit_code(&output))
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
// DECISION: D191
pub fn guard_commit(context: &Context) -> Result<i32> {
    let settings = &context.config.git;
    let repository = crate::shared::vcs::Repository::discover(&context.root)?
        .ok_or_else(|| anyhow::anyhow!("commit guard requires a VCS repository"))?;
    let (branch, merge) = repository.commit_context()?;
    let base = settings.base.as_str();
    let prefix = settings.prefix.as_str();
    let feature = branch.starts_with(prefix);
    if feature {
        // DECISION: D213
        let approved = approval::guard(context, &repository)?;
        let refused = approved != 0;
        if refused {
            return Ok(approved);
        }
        return state::State::of(context)?.guard(&repository);
    }
    let base_merge = branch == base && merge;
    if base_merge {
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
    let catalog = catalog_run(root)?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    for line in input.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let [old, new, reference] = fields.as_slice() else {
            return Ok(1);
        };
        // DECISION: D204
        let base_changed = catalog && *reference == format!("refs/heads/{}", settings.base);
        if base_changed {
            console::stderr(format!(
                "{} changes only through feature-merge",
                settings.base
            ));
            return Ok(1);
        }
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
