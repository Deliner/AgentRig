use super::{Backend, Kind, Repository, Source, git, mercurial};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(default)]
    pub backend: Backend,
    pub base: String,
    pub prefix: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            backend: Kind::Git.into(),
            base: "main".into(),
            prefix: "feature/".into(),
        }
    }
}

impl Kind {
    pub fn executable(self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::Mercurial => "hg",
        }
    }

    pub fn root_command(self) -> &'static str {
        match self {
            Self::Git => "git rev-parse --show-toplevel",
            Self::Mercurial => "hg root",
        }
    }

    pub fn hooks(self, binary: &str) -> Vec<(&'static str, String)> {
        let prefix = format!("#!/bin/sh\nset -eu\nroot=$({})\n", self.root_command());
        match self {
            Self::Git => vec![
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
            ],
            Self::Mercurial => vec![(
                "pretxncommit",
                format!(
                    "{prefix}{binary} guard-commit --root \"$root\" --revision \"$HG_NODE\"\nexec {binary} check --root \"$root\" --revision \"$HG_NODE\"\n"
                ),
            )],
        }
    }

    pub fn repository(self, root: &Path) -> Result<Option<Repository<'_>>> {
        let repository = Repository::discover(root)?;
        ensure!(
            repository.as_ref().is_none_or(|repo| repo.kind == self),
            "configured VCS does not match the existing repository; preserve it and select its backend"
        );
        Ok(repository)
    }

    pub fn initialize(self, root: &Path, base: &str) -> Result<()> {
        let missing = self.repository(root)?.is_none();
        if missing {
            match self {
                Self::Git => {
                    git::run(root, &["init", "-q", "-b", base])?;
                }
                Self::Mercurial => {
                    mercurial::initialize(root, base)?;
                }
            }
        }
        Ok(())
    }
}

impl Repository<'_> {
    pub fn hook_registration(&self) -> Result<String> {
        match self.kind {
            Kind::Git => Ok(String::from_utf8(git::run(
                self.root,
                &["config", "--default", "", "--get", "core.hooksPath"],
            )?)?
            .trim()
            .into()),
            Kind::Mercurial => mercurial::configuration(self.root, "hooks.pretxncommit.agentrig"),
        }
    }

    pub fn expected_registration(&self, directory: &str) -> String {
        match self.kind {
            Kind::Git => directory.into(),
            Kind::Mercurial => format!(
                "sh {}",
                shell_words::quote(&format!("{directory}/pretxncommit"))
            ),
        }
    }

    pub fn validate_registration(&self, directory: &str) -> Result<()> {
        Backend::Native(self.kind)
            .source(self.root)
            .validate_registration(directory)
    }

    pub fn hooks_registered(&self, directory: &str) -> Result<bool> {
        Backend::Native(self.kind)
            .source(self.root)
            .hooks_registered(directory)
    }

    fn registration_values(&self, directory: &str) -> Result<Vec<(&'static str, String, String)>> {
        let desired = self.expected_registration(directory);
        let current = self.hook_registration()?;
        match self.kind {
            Kind::Git => Ok(vec![("core.hooksPath", current, desired)]),
            Kind::Mercurial => Ok(vec![
                ("hooks.pretxncommit.agentrig", current, desired),
                ignore_registration(self.root, directory)?,
                (
                    "ui.ignore.agentrig",
                    mercurial::configuration(self.root, "ui.ignore.agentrig")?,
                    format!("{directory}.hgignore"),
                ),
            ]),
        }
    }

    pub fn register_hooks(&self, directory: &str) -> Result<()> {
        Backend::Native(self.kind)
            .source(self.root)
            .register_hooks(directory)
    }

    fn write_registration(&self, directory: &str) -> Result<()> {
        let changes = self
            .registration_values(directory)?
            .into_iter()
            .filter(|(_, current, desired)| current != desired);
        for (key, _, desired) in changes {
            match self.kind {
                Kind::Git => {
                    git::run(self.root, &["config", key, &desired])?;
                }
                Kind::Mercurial => {
                    let ignore = key == ".hgignore";
                    if ignore {
                        let mut file = fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(self.root.join(".hgignore"))?;
                        writeln!(file, "\n{desired}")?;
                        continue;
                    }
                    let mut file = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(self.root.join(".hg/hgrc"))?;
                    let (section, name) = key.split_once('.').expect("native configuration key");
                    writeln!(file, "\n[{section}]\n{name} = {desired}")?;
                }
            }
        }
        Ok(())
    }
}

fn ignore_registration(root: &Path, directory: &str) -> Result<(&'static str, String, String)> {
    let path = root.join(".hgignore");
    ensure!(
        !path.is_symlink(),
        "preserve .hgignore symlink; use a regular ignore file before setup"
    );
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    let desired = format!("include:{directory}.hgignore");
    let present = text.lines().any(|line| line == desired);
    let current = if present {
        desired.clone()
    } else {
        String::new()
    };
    Ok((".hgignore", current, desired))
}

impl Source<'_> {
    pub fn registration_values(&self, directory: &str) -> Result<Vec<(String, String, String)>> {
        match self.backend {
            Backend::Native(kind) => match kind.repository(self.root)? {
                Some(repository) => Ok(repository
                    .registration_values(directory)?
                    .into_iter()
                    .map(|(key, current, desired)| (key.into(), current, desired))
                    .collect()),
                None => Ok(Vec::new()),
            },
            Backend::External(adapter) => {
                let values: Vec<(String, String, String)> = adapter.call(
                    self.root,
                    "registration",
                    serde_json::json!({"directory": directory}),
                )?;
                let mut keys = std::collections::BTreeSet::new();
                ensure!(
                    !values.is_empty(),
                    "external VCS registration must identify managed settings"
                );
                for (key, _, desired) in &values {
                    ensure!(
                        !key.is_empty() && !desired.is_empty() && keys.insert(key),
                        "invalid or duplicate external VCS registration setting"
                    );
                }
                Ok(values)
            }
        }
    }

    pub fn validate_registration(&self, directory: &str) -> Result<()> {
        for (key, current, desired) in self.registration_values(directory)? {
            ensure!(
                current.is_empty() || current == desired,
                "setup conflict: {key}={current}; existing registration preserved"
            );
        }
        Ok(())
    }

    pub fn hooks_registered(&self, directory: &str) -> Result<bool> {
        let values = self.registration_values(directory)?;
        Ok(!values.is_empty()
            && values
                .iter()
                .all(|(_, current, desired)| current == desired))
    }

    pub fn register_hooks(&self, directory: &str) -> Result<()> {
        self.validate_registration(directory)?;
        match self.backend {
            Backend::Native(kind) => {
                Repository::new(self.root, *kind).write_registration(directory)?
            }
            Backend::External(adapter) => {
                adapter.call::<()>(
                    self.root,
                    "register-hooks",
                    serde_json::json!({"directory": directory}),
                )?;
                ensure!(
                    self.hooks_registered(directory)?,
                    "VCS hook registration did not establish the required settings; inspect and preserve current state"
                );
            }
        }
        Ok(())
    }
}
