mod init;
mod wizard;
pub use init::run as init;
mod delegation;
mod environment;
mod input;
mod preview;
mod reconcile;
pub(crate) mod registration;
mod report;
use super::{Config, Files, config, source};
use anyhow::{Result, ensure};
use std::{fs, path::Path};

pub fn inspect(root: &Path, args: &[String]) -> Result<i32> {
    ensure!(args.len() == 1, "config-inspect CONFIG_YAML [--root PATH]");
    let prepared = input::external(root, &root.join(&args[0]))?;
    preview::validate(root, &prepared.config, &prepared.files)?;
    prepared.verify()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report::configuration(
            root,
            &prepared.config,
            &prepared.files,
        )?)?
    );
    Ok(0)
}

pub(crate) fn update(root: &Path, path: &Path) -> Result<(Config, Files)> {
    init::reject_legacy(root)?;
    let mut prepared = input::external(root, path)?;
    let current = config::read(root)?;
    ensure!(
        prepared.config.runtime == current.runtime,
        "configuration update cannot change the runtime version; use a release upgrade"
    );
    ensure!(
        current.paths.service == prepared.config.paths.service
            && current.paths.runtime == prepared.config.paths.runtime,
        "configuration update must preserve service and recovery runtime locations"
    );
    registration::configure(root, &prepared.config, &mut prepared.files)?;
    preview::validate(root, &prepared.config, &prepared.files)?;
    prepared.verify()?;
    Ok((prepared.config, prepared.files))
}

pub fn run(root: &Path, args: &[String]) -> Result<i32> {
    let mut args = args.to_vec();
    let external =
        crate::arguments::take_option(&mut args, "--config")?.map(std::path::PathBuf::from);
    let preview = args == ["--preview"];
    ensure!(
        args.is_empty() || preview,
        "setup [--config CONFIG_YAML] [--preview] [--root PATH]"
    );
    let prepared = input::prepare(root, external.as_deref())?;
    let config = &prepared.config;
    let installation = prepare(root, config, prepared.files.clone())?;
    prepared.verify()?;
    if preview {
        print_preview(root, config, &installation)?;
        return Ok(0);
    }
    install(root, config, &installation)
}

pub(super) fn prepare(
    root: &Path,
    config: &Config,
    files: Files,
) -> Result<reconcile::Installation> {
    init::reject_legacy(root)?;
    ensure!(
        config.runtime == config::VERSION,
        "setup needs the pinned runtime; use upgrade for a release change"
    );
    let installed = root.join(config::FILE).is_file();
    if installed {
        crate::scaffold::recovery::guard(root)?;
    }
    let mut installation =
        reconcile::Installation::prepare(root, files, &config.paths.service_path("manifest.json"))?;
    preview::validate(root, config, &installation.files)?;
    registration::configure(root, config, &mut installation.files)?;
    Ok(installation)
}

pub(super) fn print_preview(
    root: &Path,
    config: &Config,
    installation: &reconcile::Installation,
) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&report::prepared(root, config, installation)?)?
    );
    Ok(())
}

pub(super) fn install(
    root: &Path,
    config: &Config,
    installation: &reconcile::Installation,
) -> Result<i32> {
    installation.apply(root)?;
    config.vcs.backend.initialize(root, &config.vcs.base)?;
    fs::create_dir_all(config::relative(root, &config.paths.runtime)?)?;
    if let Some(review) = &config.capabilities.review {
        let review = review_runner::config::load(&config::relative(root, &review.config)?)?;
        fs::create_dir_all(review.runner.runtime_root)?;
        fs::create_dir_all(review.runner.report_root)?;
    }
    config
        .vcs
        .backend
        .source(root)
        .register_hooks(&config.paths.service_path("hooks"))?;
    println!("Setup installed the configured worker environment. Authentication remains separate.");
    super::doctor(&config::Context::load(root)?)
}
