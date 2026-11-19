// DECISION: D010
use super::state::State;
use crate::project_config::Context as ProjectContext;
use crate::shared::vcs::{Repository, Revisions, Settings};
use anyhow::{Context, Result, ensure};

// DECISION: D194
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
        observed.operation.is_none(),
        "finish or abort the pending VCS operation before starting a feature"
    );
    repository.run(&["check-ref-format", "--branch", &branch])?;
    repository.run(&["switch", "-c", &branch])?;
    Ok(branch)
}

// DECISION: D191
pub(super) fn integrate(
    repository: &Repository,
    context: &ProjectContext,
    mut check: impl FnMut() -> Result<i32>,
    mut run: impl FnMut(&[&str]) -> Result<i32>,
) -> Result<(String, i32)> {
    let settings = &context.config.git;
    let feature = repository.observe()?.branch;
    ensure!(
        feature.starts_with(&settings.prefix),
        "integration requires a {} branch",
        settings.prefix
    );
    let state = State::of(context)?;
    clean(repository, &[".", &state.excluded()])?;
    let code = state.commit_reset(repository, &mut run)?;
    let failed = code != 0;
    if failed {
        return Ok((feature, code));
    }
    let revisions = Revisions {
        from: &settings.base,
        to: &feature,
    };
    let code = rebase_divergent(repository, revisions, &mut run)?;
    let failed = code != 0;
    if failed {
        return Ok((feature, code));
    }
    let code = check()?;
    let failed = code != 0;
    if failed {
        return Ok((feature, code));
    }
    clean(repository, &[])?;
    select_base(repository, settings, &feature)?;
    let code = run(&["merge", "--no-ff", "--no-edit", &feature])?;
    Ok((feature, code))
}

fn rebase_divergent(
    repository: &Repository,
    revisions: Revisions,
    run: &mut impl FnMut(&[&str]) -> Result<i32>,
) -> Result<i32> {
    let base = revisions.from;
    let divergent = !repository.ancestor(revisions)?;
    if divergent {
        return run(&["rebase", "--rebase-merges", base]);
    }
    Ok(0)
}

fn select_base(repository: &Repository, settings: &Settings, feature: &str) -> Result<()> {
    let base = repository.resolve(&settings.base)?;
    switch_or_restore_head(repository, &settings.base)?;
    let unchanged = repository.resolve(&settings.base)? == base
        && repository.ancestor(Revisions {
            from: &settings.base,
            to: feature,
        })?;
    let base_advanced = !unchanged;
    if base_advanced {
        switch_or_restore_head(repository, feature)?;
        anyhow::bail!("base advanced during integration; retry from the feature branch");
    }
    Ok(())
}

fn switch_or_restore_head(repository: &Repository, target: &str) -> Result<()> {
    let Err(refused) = repository.run(&["switch", target]) else {
        return Ok(());
    };
    repository
        .run(&["read-tree", "-u", "--reset", "HEAD"])
        .with_context(|| format!("{refused:#}"))?;
    Err(refused)
}

fn clean(repository: &Repository, pathspecs: &[&str]) -> Result<()> {
    let status = [&["status", "--porcelain", "--"][..], pathspecs].concat();
    ensure!(
        repository.run(&status)?.is_empty(),
        "working tree must be clean; preserve or commit pending changes before switching branches"
    );
    Ok(())
}
