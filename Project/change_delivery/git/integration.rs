// DECISION: D010
use crate::shared::vcs::{Repository, Settings};
use anyhow::{Result, ensure};
use std::process::Command;

pub(super) fn start(repository: &Repository, settings: &Settings, name: &str) -> Result<String> {
    ensure!(
        !name.is_empty() && !name.starts_with('-'),
        "feature name required"
    );
    let branch = format!("{}{name}", settings.prefix);
    let observed = repository.observe()?;
    ensure!(
        observed.branch == settings.base,
        "start from {}",
        settings.base
    );
    ensure!(
        observed.status.is_empty(),
        "working tree must be clean; preserve or commit pending changes before switching branches"
    );
    ensure!(
        !observed.merge_in_progress && !observed.rebase_in_progress,
        "finish or abort the pending VCS operation before starting a feature"
    );
    repository.run(&["check-ref-format", "--branch", &branch])?;
    repository.run(&["switch", "-c", &branch])?;
    Ok(branch)
}

pub(super) fn integrate(
    repository: &Repository,
    settings: &Settings,
    mut check: impl FnMut() -> Result<i32>,
    mut run: impl FnMut(&mut Command) -> Result<i32>,
) -> Result<(String, i32)> {
    let feature = repository.observe()?.branch;
    ensure!(
        feature.starts_with(&settings.prefix),
        "integration requires a {} branch",
        settings.prefix
    );
    clean(repository)?;
    let divergent = !repository.ancestor(&settings.base, &feature)?;
    if divergent {
        let code = run(&mut repository.command(&["rebase", "--rebase-merges", &settings.base]))?;
        let failed = code != 0;
        if failed {
            return Ok((feature, code));
        }
    }
    let code = check()?;
    let failed = code != 0;
    if failed {
        return Ok((feature, code));
    }
    clean(repository)?;
    select_base(repository, settings, &feature)?;
    let code = run(&mut repository.command(&["merge", "--no-ff", "--no-edit", &feature]))?;
    Ok((feature, code))
}

fn select_base(repository: &Repository, settings: &Settings, feature: &str) -> Result<()> {
    let base = repository.resolve(&settings.base)?;
    repository.run(&["switch", &settings.base])?;
    let unchanged = repository.resolve(&settings.base)? == base
        && repository.ancestor(&settings.base, feature)?;
    let base_advanced = !unchanged;
    if base_advanced {
        repository.run(&["switch", feature])?;
        anyhow::bail!("base advanced during integration; retry from the feature branch");
    }
    Ok(())
}

fn clean(repository: &Repository) -> Result<()> {
    ensure!(
        repository.run(&["status", "--porcelain"])?.is_empty(),
        "working tree must be clean; preserve or commit pending changes before switching branches"
    );
    Ok(())
}
