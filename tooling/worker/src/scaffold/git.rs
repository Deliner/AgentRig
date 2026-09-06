// DECISION: D010
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
    let code = prepare_integration(context, &feature)?;
    let failed = code != 0;
    if failed {
        return Ok(code);
    }
    clean(root)?;
    let base = git(root, &["rev-parse", &settings.base])?;
    git(root, &["switch", &settings.base])?;
    let unchanged = git(root, &["rev-parse", &settings.base])? == base
        && ancestor(root, &settings.base, &feature)?;
    let base_advanced = !unchanged;
    if base_advanced {
        git(root, &["switch", &feature])?;
        anyhow::bail!("base advanced during integration; retry from the feature branch");
    }
    let code = merge_branch(root, &feature)?;
    let succeeded = code == 0;
    if succeeded {
        return cleanup_merged(context, &feature);
    }
    Ok(code)
}
fn cleanup_merged(context: &Context, feature: &str) -> Result<i32> {
    let identified = agentrig::jobs::owner().is_some();
    let anonymous = !identified;
    if anonymous {
        return Ok(0);
    }
    let result =
        agentrig::jobs::cleanup::run(&context.path(&context.config.paths.runtime)?, Some(feature));
    match result {
        Ok(report) => {
            let failed = report["errors"]
                .as_array()
                .is_some_and(|errors| !errors.is_empty());
            println!("post-merge cleanup: {report}");
            if failed {
                eprintln!("merge succeeded; cleanup failed. Retry job-cleanup --branch {feature}");
            }
            Ok(i32::from(failed))
        }
        Err(error) => {
            eprintln!(
                "merge succeeded; cleanup failed: {error}. Retry job-cleanup --branch {feature}"
            );
            Ok(1)
        }
    }
}
fn merge_branch(root: &Path, feature: &str) -> Result<i32> {
    let code = super::process::run(
        root,
        &[
            "git".into(),
            "merge".into(),
            "--no-ff".into(),
            "--no-edit".into(),
            feature.to_owned(),
        ],
        false,
    )?;
    let merged = code == 0;
    if merged {
        println!("merged {feature}; branch retained");
    }
    Ok(code)
}

fn prepare_integration(context: &Context, feature: &str) -> Result<i32> {
    let root = &context.root;
    let base = &context.config.git.base;
    let divergent = !ancestor(root, base, feature)?;
    if divergent {
        let code = super::process::run(
            root,
            &[
                "git".into(),
                "rebase".into(),
                "--rebase-merges".into(),
                base.clone(),
            ],
            false,
        )?;
        let failed = code != 0;
        if failed {
            return Ok(code);
        }
    }
    gate::run(root, false)
}
