mod delegation;
mod input;
mod preview;
mod reconcile;
mod registration;
mod report;
use super::{Config, Files, config, manifest};
use anyhow::{Result, ensure};
use std::{fs, path::Path};

pub fn run(root: &Path, args: &[String]) -> Result<i32> {
    let mut args = args.to_vec();
    let external = crate::util::take_option(&mut args, "--config")?.map(std::path::PathBuf::from);
    let preview = args == ["--preview"];
    ensure!(
        args.is_empty() || preview,
        "setup [--config CONFIG_YAML] [--preview] [--root PATH]"
    );
    let prepared = input::prepare(root, external.as_deref())?;
    let config = &prepared.config;
    ensure!(
        config.runtime == config::VERSION,
        "setup needs the pinned runtime; use upgrade for a release change"
    );
    let installed = root.join(config::FILE).is_file();
    if installed {
        crate::scaffold::upgrade::recovery::guard(root)?;
    }
    let files = prepared.files.clone();
    let mut installation =
        reconcile::Installation::prepare(root, files, &config.paths.service_path("manifest.json"))?;
    preview::validate(root, config, &installation.files)?;
    registration::configure(root, config, &mut installation.files)?;
    prepared.verify()?;
    if preview {
        println!(
            "{}",
            serde_json::to_string_pretty(&report::prepared(root, config, &installation)?)?
        );
        return Ok(0);
    }
    install(root, config, &installation)
}

fn source(root: &Path, files: &Files, path: &str) -> Result<String> {
    match files.get(path) {
        Some(bytes) => Ok(std::str::from_utf8(bytes)?.into()),
        None => Ok(fs::read_to_string(config::relative(root, path)?)?),
    }
}

fn install(root: &Path, config: &Config, installation: &reconcile::Installation) -> Result<i32> {
    let fresh_git = !root.join(".git").exists();
    if fresh_git {
        crate::util::git(root, &["init", "-q", "-b", &config.git.base])?;
    }
    installation.apply(root)?;
    fs::create_dir_all(config::relative(root, &config.paths.runtime)?)?;
    if let Some(review) = &config.capabilities.review {
        let review = review_runner::config::load(&config::relative(root, &review.config)?)?;
        fs::create_dir_all(review.runner.runtime_root)?;
        fs::create_dir_all(review.runner.report_root)?;
    }
    crate::util::git(
        root,
        &[
            "config",
            "core.hooksPath",
            &config.paths.service_path("hooks"),
        ],
    )?;
    println!("Setup installed the configured worker environment. Authentication remains separate.");
    super::doctor(&config::Context::load(root)?)
}
