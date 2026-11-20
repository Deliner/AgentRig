// DECISION: D218
use crate::command_runner;
use crate::project_config::Context;
use crate::shared::atomic_files::Overlay;
use crate::shared::console;
use crate::shared::vcs::{Repository, Revisions};
use anyhow::{Context as _, Result, ensure};
use std::path::Path;

pub(super) fn propose(context: &Context, reference: &str) -> Result<i32> {
    let pin = context
        .protection
        .pin()
        .context("propose-pin needs the setting git.pin")?;
    let repository =
        Repository::discover(&context.root)?.context("propose-pin requires a VCS repository")?;
    let revision = repository.resolve(reference)?;
    let pinned = descending(&repository, pin, &revision)?;
    let mirror = super::proposal::mirror(context)?;
    let overlay = Overlay {
        mirror: &mirror,
        root: &context.root,
    };
    overlay.propose(pin, format!("{revision}\n").as_bytes())?;
    let revisions = Revisions {
        from: &pinned,
        to: &revision,
    };
    let listed = listed(&sources(context, &repository, revisions)?);
    console::stdout(format!(
        "Prepared the pin {revision}.\nSource files changed since the pinned {pinned}:\n  {listed}\nAsk the user to type `! just approve` in Claude Code or to run `just approve` in a terminal: it installs {revision} and only then pins it."
    ));
    Ok(0)
}

pub(super) fn install(
    context: &Context,
    repository: &Repository<'_>,
    overlay: &Overlay<'_>,
) -> Result<()> {
    let pin = context.protection.pin();
    let installer = context.config.git.install.as_deref();
    let (Some(pin), Some(_)) = (pin, installer) else {
        return Ok(());
    };
    let Some(proposed) = overlay.proposed(pin)? else {
        return Ok(());
    };
    let proposed = String::from_utf8(proposed)?;
    let proposed = proposed.trim();
    let revision = repository.resolve(proposed)?;
    ensure!(
        revision == proposed,
        "the proposed pin {proposed} must be a full commit hash"
    );
    descending(repository, pin, &revision)?;
    install_revision(context, &revision)
}

// DECISION: D226
pub(super) fn install_revision(context: &Context, revision: &str) -> Result<()> {
    let Some(installer) = context.config.git.install.as_deref() else {
        return Ok(());
    };
    let code = command_runner::run(context, installer, &[revision.to_owned()])?;
    ensure!(
        code == 0,
        "installing {revision} failed with exit code {code}; the pin is unchanged"
    );
    Ok(())
}

pub(super) fn descending(
    repository: &Repository<'_>,
    pin: &Path,
    revision: &str,
) -> Result<String> {
    let committed = String::from_utf8(repository.read("HEAD", pin)?)?;
    let pinned = committed.trim();
    let revisions = Revisions {
        from: pinned,
        to: revision,
    };
    let forward = repository.ancestor(revisions)?;
    ensure!(
        forward,
        "revision {revision} does not descend from the pinned {pinned}"
    );
    Ok(pinned.to_owned())
}

// DECISION: D226
pub(super) fn sources(
    context: &Context,
    repository: &Repository<'_>,
    revisions: Revisions<'_>,
) -> Result<Vec<String>> {
    let sources = context.config.paths.source_matcher()?;
    Ok(repository
        .changed_paths(revisions)?
        .into_iter()
        .filter(|path| sources.is_match(path))
        .collect())
}

// DECISION: D226
pub(super) fn listed(sources: &[String]) -> String {
    let unchanged = sources.is_empty();
    if unchanged {
        "none".to_owned()
    } else {
        sources.join("\n  ")
    }
}
