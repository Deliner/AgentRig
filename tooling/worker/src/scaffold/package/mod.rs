pub(super) mod adapters;
mod assets;
mod claude;
mod doctor;
mod executors;
mod lint;
pub(super) mod manifest;
mod review;
pub(super) mod setup;
mod template;
use super::config::{self, Config};
use anyhow::{Result, ensure};
pub use doctor::run as doctor;
use std::{collections::BTreeMap, fs, path::Path};
type Files = BTreeMap<String, Vec<u8>>;
fn bundle(root: &Path, config: &Config) -> Result<Files> {
    let mut files = guidance(config);
    files.insert(
        config::FILE.into(),
        review_runner::config::yaml::encode(config)?.into_bytes(),
    );
    for (name, source) in assets::memory() {
        files.insert(
            format!("{}/{name}.md", config.paths.memory),
            source.into_bytes(),
        );
    }
    add_policy(&mut files, config)?;
    add_runtime(root, &mut files, config)?;
    review::bundle(&mut files, config)?;
    files.insert(
        config.paths.service_path("manifest.json"),
        manifest::installed(&files, config)?,
    );
    Ok(files)
}

pub(super) fn guidance(config: &Config) -> Files {
    let mut files: Files = assets::skills(config)
        .into_iter()
        .map(|(name, source)| {
            (
                format!("{}/{name}/SKILL.md", config.paths.skills),
                source.into_bytes(),
            )
        })
        .collect();
    files.insert(
        "AGENTS.md".into(),
        assets::instructions(config).into_bytes(),
    );
    files
}
fn add_policy(files: &mut Files, config: &Config) -> Result<()> {
    if config.capabilities.lint {
        files.insert(
            config.paths.lint.clone(),
            lint::template(config)?.into_bytes(),
        );
    }
    if let Some(path) = &config.hooks.reminder {
        files.insert(
            path.clone(),
            include_bytes!("../../../assets/skills/complexity-discipline/context-reminder.json")
                .to_vec(),
        );
    }
    Ok(())
}
fn add_runtime(root: &Path, files: &mut Files, config: &Config) -> Result<()> {
    let generated = adapters::generated(root, config)?;
    files.insert(
        config.paths.service_path(".gitignore"),
        b"runtime/\n/inputs/runtime/\n/inputs/reports/\n".to_vec(),
    );
    files.insert(
        config.paths.service_path("bin/agentrig"),
        fs::read(std::env::current_exe()?)?,
    );
    files.insert("justfile".into(), template::justfile(config).into_bytes());
    client_files(files, config, &generated.root_command)?;
    for (path, contents) in generated.files {
        ensure!(
            !files.contains_key(&path),
            "generated VCS file conflicts with another resource: {path}"
        );
        files.insert(path, contents.into_bytes());
    }
    Ok(())
}
fn client_files(files: &mut Files, config: &Config, root_command: &str) -> Result<()> {
    match config.frontend {
        config::Frontend::Codex => {
            files.insert(
                ".codex/config.toml".into(),
                adapters::CODEX_CONFIG.as_bytes().to_vec(),
            );
            files.insert(
                ".codex/hooks.json".into(),
                adapters::registration(config, root_command)?,
            );
        }
        config::Frontend::ClaudeCode => claude::bundle(files, config, root_command)?,
    }
    Ok(())
}

fn source(root: &Path, files: &Files, path: &str) -> Result<String> {
    match files.get(path) {
        Some(bytes) => Ok(std::str::from_utf8(bytes)?.into()),
        None => Ok(fs::read_to_string(config::relative(root, path)?)?),
    }
}
