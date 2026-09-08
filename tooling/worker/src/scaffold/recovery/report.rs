use super::{
    model::{Change, Plan},
    storage,
};
use anyhow::{Result, ensure};
use std::{fs, path::Path, process::Command};

pub fn run(directory: &Path, plan: &Plan) -> Result<()> {
    let mut report = format!(
        "Upgrade {} -> {} (baseline: {})\n",
        plan.from_version, plan.to_version, plan.baseline
    );
    for (path, change) in &plan.files {
        let action = serde_json::to_string(&change.action)?;
        report.push_str(&format!("{action} {path}: {}\n", change.reason));
        let changed = change.before != change.after;
        if changed {
            report.push_str(&diff(directory, change)?);
        }
    }
    report.push_str(&format!("Checks: {}\n", plan.checks.join(", ")));
    fs::write(directory.join("diff.txt"), &report)?;
    print!("{report}");
    Ok(())
}
fn diff(directory: &Path, change: &Change) -> Result<String> {
    let left = directory.join("before");
    let right = directory.join("after");
    for (path, state) in [(&left, &change.before), (&right, &change.after)] {
        let bytes = match &state.sha256 {
            Some(hash) => storage::payload(directory, hash)?,
            None => Vec::new(),
        };
        fs::write(path, bytes)?;
    }
    let output = Command::new("git")
        .args(["diff", "--no-index", "--no-ext-diff", "--no-color"])
        .arg(&left)
        .arg(&right)
        .output()?;
    ensure!(
        matches!(output.status.code(), Some(0 | 1)),
        "cannot produce upgrade diff"
    );
    fs::remove_file(left)?;
    fs::remove_file(right)?;
    Ok(format!(
        "mode {:?} -> {:?}\n{}",
        change.before.mode,
        change.after.mode,
        String::from_utf8_lossy(&output.stdout)
    ))
}
