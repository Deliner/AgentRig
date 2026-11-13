use crate::command_runner::jobs::Capture;
use crate::command_runner::sandbox::{self, Sandboxing};
use crate::project_config::Access;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fmt, path::Path};

// DECISION: D174
const RIG_TOOLCHAIN: &str = include_str!("../../../rust-toolchain.toml");
const UPDATE: &str =
    "run `rustup update` or raise the channel in the project's rust-toolchain.toml";
// DECISION: D175
const CAPABILITIES: &str = include_str!("capabilities.txt");

#[derive(Deserialize)]
struct ToolchainFile {
    toolchain: Channel,
}

#[derive(Deserialize)]
struct Channel {
    channel: String,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    // DECISION: D174
    pub fn rig() -> Result<Self> {
        let file: ToolchainFile =
            toml::from_str(RIG_TOOLCHAIN).context("parse the rig's rust-toolchain.toml")?;
        let channel = file.toolchain.channel;
        Self::parse(&channel)
            .with_context(|| format!("the rig's toolchain channel {channel} is not a version"))
    }

    // DECISION: D175
    pub fn capabilities(self) -> String {
        let Self { major, minor, .. } = self;
        format!("## Rust {major}.{minor}\n\n{CAPABILITIES}")
    }

    fn parse(text: &str) -> Option<Self> {
        let release = text.split_once('-').map_or(text, |(release, _)| release);
        let mut parts = release.split('.').map(str::parse::<u32>);
        let version = Self {
            major: parts.next()?.ok()?,
            minor: parts.next()?.ok()?,
            patch: parts.next()?.ok()?,
        };
        parts.next().is_none().then_some(version)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            major,
            minor,
            patch,
        } = self;
        write!(formatter, "{major}.{minor}.{patch}")
    }
}

enum Found {
    Meets(Version),
    Older(Version),
    Missing,
}

pub struct Requirement {
    required: Version,
    found: Found,
}

impl Requirement {
    // DECISION: D174
    pub fn inspect(root: &Path) -> Result<Self> {
        let required = Version::rig()?;
        let found = match project_rustc(root) {
            Some(version) if version >= required => Found::Meets(version),
            Some(version) => Found::Older(version),
            None => Found::Missing,
        };
        Ok(Self { required, found })
    }

    pub fn met(&self) -> bool {
        match self.found {
            Found::Meets(_) => true,
            Found::Older(_) | Found::Missing => false,
        }
    }
}

impl fmt::Display for Requirement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let required = self.required;
        match self.found {
            Found::Meets(found) => write!(formatter, "found {found}, requires {required} or newer"),
            Found::Older(found) => write!(
                formatter,
                "found {found}, requires {required} or newer; {UPDATE}"
            ),
            Found::Missing => write!(
                formatter,
                "found no rustc version, requires {required} or newer; {UPDATE}"
            ),
        }
    }
}

// DECISION: D174
fn project_rustc(root: &Path) -> Option<Version> {
    let directory = root.ancestors().find(|directory| directory.is_dir())?;
    let argv = ["rustc", "--version"].map(String::from);
    let options = Sandboxing {
        access: Access::Writable,
        capture: Capture::Collect,
        cargo_target: None,
    };
    let output = sandbox::tracked(directory, &argv, options, None).ok()?;
    let printed = output.status.success().then_some(output.stdout)?;
    let printed = String::from_utf8(printed).ok()?;
    Version::parse(printed.split_whitespace().nth(1)?)
}
