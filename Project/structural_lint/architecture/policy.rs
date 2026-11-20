// DECISION: D028
// DECISION: D043
// DECISION: D044
use crate::shared::fs_with_context as fs;
use crate::structural_lint::rule_identities::contract_settings::FILENAME;
use anyhow::{Result, ensure};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

// DECISION: D070
const PLACEHOLDER_PURPOSES: &[&str] = &[
    "todo",
    "tbd",
    "fixme",
    "wip",
    "xxx",
    "na",
    "none",
    "placeholder",
    "description",
    "purpose",
    "module",
    "directory",
    "package",
    "folder",
    "file",
    "code",
    "stuff",
    "misc",
    "various",
    "temporary",
];

// DECISION: D078
#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Zone {
    Pure,
}

// DECISION: D079
const DEFAULT_DEPENDENTS: usize = 6;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Declaration {
    purpose: String,
    #[serde(default)]
    files: BTreeMap<String, String>,
    #[serde(default)]
    directories: BTreeMap<String, String>,
    #[serde(default, rename = "type")]
    kind: Option<Kind>,
    // DECISION: D078
    #[serde(default)]
    zone: Option<Zone>,
    // DECISION: D079
    #[serde(default)]
    dependents: Option<usize>,
    // DECISION: D228
    #[serde(default)]
    group: bool,
}
// DECISION: D129
impl Declaration {
    pub(super) fn files(&self) -> &BTreeMap<String, String> {
        &self.files
    }
    pub(super) fn directories(&self) -> &BTreeMap<String, String> {
        &self.directories
    }
    pub(super) fn kind(&self) -> Option<Kind> {
        self.kind
    }
    pub(super) fn zone(&self) -> Option<Zone> {
        self.zone
    }
    pub(super) fn dependents(&self) -> usize {
        self.dependents.unwrap_or(DEFAULT_DEPENDENTS)
    }
    // DECISION: D228
    pub(super) fn group(&self) -> bool {
        self.group
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Flat,
    Layered,
    Hexagonal,
    Onion,
    Clean,
    Custom,
}

impl Kind {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::Layered => "layered",
            Self::Hexagonal => "hexagonal",
            Self::Onion => "onion",
            Self::Clean => "clean",
            Self::Custom => "custom",
        }
    }

    pub(super) fn parts(self) -> Option<&'static [&'static str]> {
        match self {
            Self::Flat => Some(&[]),
            Self::Layered => Some(&["presentation", "application", "domain", "infrastructure"]),
            Self::Hexagonal => Some(&["adapters", "ports", "domain"]),
            Self::Onion => Some(&["infrastructure", "application", "domain"]),
            Self::Clean => Some(&["frameworks", "adapters", "use_cases", "entities"]),
            Self::Custom => None,
        }
    }
}

pub struct Description {
    pub purpose: String,
    pub role: String,
}

#[derive(Clone, Copy)]
pub struct Rooted<'a> {
    pub root: &'a Path,
    pub path: &'a Path,
}

impl Rooted<'_> {
    pub(super) fn absolute(self) -> PathBuf {
        self.root.join(self.path)
    }

    pub fn describe(self) -> Option<Description> {
        let root = self.root;
        let path = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let declaration = Rooted { root, path }.declaration().ok()?;
        let role = declaration
            .files
            .get(self.path.file_name()?.to_str()?)?
            .clone();
        Some(Description {
            purpose: declaration.purpose,
            role,
        })
    }

    pub(super) fn declaration(self) -> Result<Declaration> {
        let Self {
            root,
            path: directory,
        } = self;
        ensure!(
            directory == Path::new(".") || relative(directory),
            "invalid directory path"
        );
        let path = self.absolute().join(FILENAME);
        ensure!(
            fs::symlink_metadata(&path)?.is_file(),
            "contract must be a regular file"
        );
        ensure!(
            fs::canonicalize(&path)?.starts_with(fs::canonicalize(root)?),
            "contract escapes project root"
        );
        let declaration: Declaration = crate::shared::strict_yaml::read(&path)?;
        let purpose = declaration.purpose.trim();
        ensure!(
            !purpose.is_empty() && !declaration.purpose.contains(['\n', '\r']),
            "purpose must describe the directory responsibility on one nonempty line"
        );
        let gist = purpose
            .trim_matches(|character: char| !character.is_alphanumeric())
            .to_ascii_lowercase();
        ensure!(
            !gist.is_empty() && !PLACEHOLDER_PURPOSES.contains(&gist.as_str()),
            "purpose must state the real directory responsibility, not a placeholder"
        );
        Ok(declaration)
    }
}

fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            // DECISION: D156 variant-equality
            .all(|part| matches!(part, Component::Normal(_)))
}
