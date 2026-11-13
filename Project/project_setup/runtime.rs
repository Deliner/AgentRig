use super::Files;
use crate::project_config::{Config, Context};
use crate::shared::console;
use crate::shared::fs_with_context::{self as fs, Permissions, PermissionsExt};
use anyhow::Result;
use std::{io::Write, path::Path};

const LAUNCHER: &str = r#"#!/bin/sh
set -eu
system=$(uname -sm)
case $system in
Darwin\ *) os=macos ;;
Linux\ *) os=linux ;;
*) os=${system%% *} ;;
esac
platform=$os-${system#* }
runtime=${0%/*}/$platform/agentrig
if [ -x "$runtime" ]; then exec "$runtime" "$@"; fi
echo "agentrig: no runtime for $platform in ${0%/*}" >&2
echo "add it from the project root with an AgentRig built for $platform: agentrig runtime-add" >&2
exit 2
"#;

// DECISION: D150
pub(super) fn files(config: &Config) -> Result<Files> {
    let arch = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "arm64",
        (_, arch) => arch,
    };
    let platform = format!("{}-{arch}", std::env::consts::OS);
    Ok(Files::from([
        (
            config.paths.service_path("bin/agentrig"),
            LAUNCHER.as_bytes().to_vec(),
        ),
        (
            config
                .paths
                .service_path(&format!("bin/{platform}/agentrig")),
            fs::read(std::env::current_exe()?)?,
        ),
    ]))
}

// DECISION: D150
pub fn add(context: &Context) -> Result<i32> {
    for (path, contents) in files(&context.config)? {
        replace(&context.root.join(&path), &contents)?;
        console::stdout(format!("installed {path}"));
    }
    Ok(0)
}

fn replace(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("runtime path needs a parent"))?;
    fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(contents)?;
    temporary
        .as_file()
        .set_permissions(Permissions::from_mode(0o755))?;
    temporary.persist(path)?;
    Ok(())
}
