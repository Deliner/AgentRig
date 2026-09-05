use super::{config::Context, gate};
use crate::util::git;
use anyhow::{Result, ensure};
use std::{path::Path, process::Command};

fn clean(root: &Path) -> Result<()> {
    ensure!(
        git(root, &["status", "--porcelain"])?.is_empty(),
        "working tree must be clean; preserve or commit pending changes before switching branches"
    );
    Ok(())
}
fn ancestor(root: &Path, older: &str, newer: &str) -> Result<bool> {
    Ok(Command::new("git")
        .args(["merge-base", "--is-ancestor", older, newer])
        .current_dir(root)
        .status()?
        .success())
}
pub fn start(context: &Context, name: &str) -> Result<i32> {
    let root = &context.root;
    let settings = &context.config.git;
    let branch = format!("{}{name}", settings.prefix);
    ensure!(
        !name.is_empty() && !name.starts_with('-'),
        "feature name required"
    );
    git(root, &["check-ref-format", "--branch", &branch])?;
    ensure!(
        git(root, &["branch", "--show-current"])? == settings.base,
        "start from {}",
        settings.base
    );
    clean(root)?;
    git(root, &["switch", "-c", &branch])?;
    println!("{branch}");
    Ok(0)
}
pub fn merge(context: &Context) -> Result<i32> {
    let root = &context.root;
    let settings = &context.config.git;
    let feature = git(root, &["branch", "--show-current"])?;
    ensure!(
        feature.starts_with(&settings.prefix),
        "integration requires a {} branch",
        settings.prefix
    );
    clean(root)?;
    if !ancestor(root, &settings.base, &feature)? {
        let code = super::process::run(
            root,
            &[
                "git".into(),
                "rebase".into(),
                "--rebase-merges".into(),
                settings.base.clone(),
            ],
            false,
        )?;
        if code != 0 {
            return Ok(code);
        }
    }
    let code = gate::run(root, false)?;
    if code != 0 {
        return Ok(code);
    }
    clean(root)?;
    let base = git(root, &["rev-parse", &settings.base])?;
    git(root, &["switch", &settings.base])?;
    let unchanged = git(root, &["rev-parse", &settings.base])? == base
        && ancestor(root, &settings.base, &feature)?;
    if !unchanged {
        git(root, &["switch", &feature])?;
        anyhow::bail!("base advanced during integration; retry from the feature branch");
    }
    let code = super::process::run(
        root,
        &[
            "git".into(),
            "merge".into(),
            "--no-ff".into(),
            "--no-edit".into(),
            feature.clone(),
        ],
        false,
    )?;
    if code == 0 {
        println!("merged {feature}; branch retained");
    }
    Ok(code)
}
