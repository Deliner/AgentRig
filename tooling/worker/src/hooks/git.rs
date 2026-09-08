// DECISION: D019
use anyhow::Result;
use review_runner::vcs::git_context as git;
use std::{
    io::{self, Read},
    path::Path,
    process::Command,
};

// DECISION: D015
pub fn guard_commit_with(
    root: &Path,
    settings: &review_runner::vcs::Settings,
    revision: Option<&str>,
) -> Result<i32> {
    let repository = settings
        .backend
        .repository_source(root)?
        .ok_or_else(|| anyhow::anyhow!("commit guard requires a VCS repository"))?;
    let (branch, merge) = repository.commit_context(revision)?;
    let base = settings.base.as_str();
    let prefix = settings.prefix.as_str();
    let permitted = branch.starts_with(prefix) || (branch == base && merge);
    if permitted {
        return Ok(0);
    }
    eprintln!("direct commits on {base} are prohibited; create a {prefix} branch");
    Ok(1)
}
pub fn guard_reference_with(root: &Path, phase: &str, base: &str, prefix: &str) -> Result<i32> {
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
                && reference.starts_with(&format!("refs/heads/{prefix}"));
            if deletion {
                let merged = merged_reference(root, old, reference, base)?;
                if merged {
                    eprintln!("merged feature branches must be retained");
                    return Ok(1);
                }
            }
        }
    }
    Ok(0)
}

fn merged_reference(root: &Path, old: &str, reference: &str, base: &str) -> Result<bool> {
    let unknown_tip = old.chars().all(|ch| ch == '0');
    let tip = if unknown_tip {
        match git(
            root,
            &["rev-parse", "--verify", &format!("{reference}^{{commit}}")],
        ) {
            Ok(tip) => tip,
            Err(_) => return Ok(false),
        }
    } else {
        old.to_owned()
    };
    Ok(Command::new("git")
        .args(["merge-base", "--is-ancestor", &tip, base])
        .current_dir(root)
        .status()?
        .success())
}
