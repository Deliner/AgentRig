// DECISION: D019
use crate::util::git;
use anyhow::Result;
use std::{
    io::{self, Read},
    path::Path,
    process::Command,
};

// DECISION: D015
pub fn guard_commit(root: &Path) -> Result<i32> {
    let branch = git(root, &["branch", "--show-current"])?;
    let merge = root
        .join(git(root, &["rev-parse", "--git-path", "MERGE_HEAD"])?)
        .exists();
    if branch.starts_with("feature/") || (branch == "master" && merge) {
        return Ok(0);
    }
    eprintln!("direct commits on master are prohibited; use just feature-start");
    Ok(1)
}
pub fn guard_reference(root: &Path, phase: &str) -> Result<i32> {
    if phase != "prepared" {
        return Ok(0);
    }
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    for line in input.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() != 3 {
            return Ok(1);
        }
        let [old, new, reference] = [fields[0], fields[1], fields[2]];
        if new.chars().all(|ch| ch == '0') && reference.starts_with("refs/heads/feature/") {
            let tip = if old.chars().all(|ch| ch == '0') {
                match git(
                    root,
                    &["rev-parse", "--verify", &format!("{reference}^{{commit}}")],
                ) {
                    Ok(tip) => tip,
                    Err(_) => continue,
                }
            } else {
                old.to_owned()
            };
            if Command::new("git")
                .args(["merge-base", "--is-ancestor", &tip, "master"])
                .current_dir(root)
                .status()?
                .success()
            {
                eprintln!("merged feature branches must be retained");
                return Ok(1);
            }
        }
    }
    Ok(0)
}
