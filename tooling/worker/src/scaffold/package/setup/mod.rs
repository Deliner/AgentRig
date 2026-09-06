mod delegation;
mod preview;
mod reconcile;
mod registration;
use super::{Config, Files, config, manifest};
use anyhow::{Result, ensure};
use std::{fs, path::Path};

pub fn run(root: &Path, args: &[String]) -> Result<i32> {
    ensure!(
        args.is_empty(),
        "setup takes no arguments except --root PATH"
    );
    let config = config::read(root)?;
    ensure!(
        config.runtime == config::VERSION,
        "setup needs the pinned runtime; use upgrade for a release change"
    );
    crate::scaffold::upgrade::recovery::guard(root)?;
    let files = super::bundle(&config)?;
    let mut installation = reconcile::Installation::prepare(root, files)?;
    preview::validate(root, &config, &installation.files)?;
    registration::configure(root, &config, &mut installation.files)?;
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
    crate::util::git(root, &["config", "core.hooksPath", ".worker/hooks"])?;
    println!("Setup installed the configured worker environment. Authentication remains separate.");
    super::doctor(&config::Context::load(root)?)
}
