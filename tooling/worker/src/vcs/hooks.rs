use super::{Repository, export, git};
use anyhow::{Result, ensure};
use std::collections::BTreeMap;

const ROOT_COMMAND: &str = "git rev-parse --show-toplevel";

pub struct Generation<'a> {
    pub binary: &'a str,
    pub directory: &'a str,
}

pub struct Generated {
    pub root_command: String,
    pub files: BTreeMap<String, String>,
}

pub fn generate(request: &Generation<'_>) -> Result<Generated> {
    let prefix = format!("#!/bin/sh\nset -eu\nroot=$({ROOT_COMMAND})\n");
    let binary = request.binary;
    let files: BTreeMap<_, _> = [
        (
            "pre-commit",
            format!(
                "{prefix}{binary} guard-commit --root \"$root\"\nexec {binary} check --root \"$root\" --staged\n"
            ),
        ),
        (
            "reference-transaction",
            format!("{prefix}exec {binary} guard-reference --root \"$root\" \"$1\"\n"),
        ),
    ]
    .into_iter()
    .map(|(name, contents)| (format!("{}/{name}", request.directory), contents))
    .collect();
    for path in files.keys() {
        export::validate_path(path)?;
    }
    Ok(Generated {
        root_command: ROOT_COMMAND.into(),
        files,
    })
}

impl Repository<'_> {
    pub fn hook_registration(&self) -> Result<String> {
        Ok(String::from_utf8(git::run(
            self.root,
            &["config", "--default", "", "--get", "core.hooksPath"],
        )?)?
        .trim()
        .into())
    }

    pub fn validate_registration(&self, directory: &str) -> Result<()> {
        let current = self.hook_registration()?;
        ensure!(
            current.is_empty() || current == directory,
            "setup conflict: core.hooksPath={current}; existing registration preserved"
        );
        Ok(())
    }

    pub fn hooks_registered(&self, directory: &str) -> Result<bool> {
        Ok(self.hook_registration()? == directory)
    }

    pub fn register_hooks(&self, directory: &str) -> Result<()> {
        self.validate_registration(directory)?;
        let missing = !self.hooks_registered(directory)?;
        if missing {
            git::run(self.root, &["config", "core.hooksPath", directory])?;
        }
        Ok(())
    }
}
