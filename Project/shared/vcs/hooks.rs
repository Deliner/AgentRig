use super::repository::Repository;
use super::{export, git};
use anyhow::{Result, ensure};
use std::collections::BTreeMap;

pub struct Generation<'a> {
    pub binary: &'a str,
    pub directory: &'a str,
}

// DECISION: D179
// DECISION: D191
pub fn generate(request: &Generation<'_>) -> Result<BTreeMap<String, String>> {
    let prefix = "#!/bin/sh\nset -eu\n";
    let binary = request.binary;
    let files: BTreeMap<_, _> = [
        (
            "pre-commit",
            format!("{prefix}{binary} guard-commit\nexec {binary} check --staged\n"),
        ),
        (
            "reference-transaction",
            format!("{prefix}exec {binary} guard-reference \"$1\"\n"),
        ),
        (
            "post-commit",
            format!("{prefix}{binary} state-reset\nexec {binary} feature-push\n"),
        ),
    ]
    .into_iter()
    .map(|(name, contents)| (format!("{}/{name}", request.directory), contents))
    .collect();
    for path in files.keys() {
        export::validate_path(path)?;
    }
    Ok(files)
}

impl Repository<'_> {
    fn hook_registration(&self) -> Result<String> {
        Ok(String::from_utf8(git::run(
            self.root,
            &["config", "--default", "", "--get", "core.hooksPath"],
        )?)?
        .trim()
        .into())
    }

    pub fn compatible_registration(&self, directory: &str) -> Result<String> {
        let current = self.hook_registration()?;
        ensure!(
            current.is_empty() || current == directory,
            "setup conflict: core.hooksPath={current}; existing registration preserved"
        );
        Ok(current)
    }

    pub fn hooks_registered(&self, directory: &str) -> Result<bool> {
        Ok(self.hook_registration()? == directory)
    }

    pub fn register_hooks(&self, directory: &str) -> Result<()> {
        let missing = self.compatible_registration(directory)?.is_empty();
        if missing {
            git::run(self.root, &["config", "core.hooksPath", directory])?;
        }
        Ok(())
    }
}
