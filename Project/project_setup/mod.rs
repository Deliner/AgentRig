pub(super) mod adapters;
mod assets;
mod cargo;
mod claude;
mod clippy;
mod coverage;
mod doctor;
pub(super) mod initialization;
mod lint;
mod runtime;
mod skills;
mod template;
use crate::project_config::{self, Config};
use anyhow::{Result, ensure};
pub use doctor::{config_check, run as doctor};
pub use runtime::add as runtime_add;
use std::collections::BTreeMap;
type Files = BTreeMap<String, Vec<u8>>;
fn bundle(scaffold: &template::Scaffold) -> Result<Files> {
    let config = &scaffold.config;
    let mut files = guidance(config);
    for file in project_config::files(config)? {
        files.insert(file.path, file.source.into_bytes());
    }
    for (name, source) in assets::memory()? {
        files.insert(
            format!("{}/{name}.md", config.paths.memory),
            source.into_bytes(),
        );
    }
    add_policy(&mut files, config, &scaffold.rust)?;
    add_runtime(&mut files, config)?;
    Ok(files)
}

pub(super) fn guidance(config: &Config) -> Files {
    let mut files: Files = skills::bodies()
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
fn add_policy(files: &mut Files, config: &Config, rust: &cargo::Layout) -> Result<()> {
    if config.capabilities.lint {
        files.insert(
            config.paths.lint.clone(),
            lint::template(config, rust)?.into_bytes(),
        );
    }
    if let Some(path) = &config.hooks.reminder {
        files.insert(
            path.clone(),
            include_bytes!("../shared/skills/complexity-discipline/context-reminder.json").to_vec(),
        );
    }
    Ok(())
}
fn add_runtime(files: &mut Files, config: &Config) -> Result<()> {
    let generated = adapters::generated(config)?;
    files.insert(
        config.paths.service_path(".gitignore"),
        b"runtime/\n".to_vec(),
    );
    files.extend(runtime::files(config)?);
    for justfile in template::justfiles(config) {
        files.insert(justfile.path, justfile.source.into_bytes());
    }
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
        project_config::Frontend::Codex => {
            files.insert(
                ".codex/config.toml".into(),
                adapters::CODEX_CONFIG.as_bytes().to_vec(),
            );
            files.insert(
                ".codex/hooks.json".into(),
                adapters::registration(config, root_command)?,
            );
        }
        project_config::Frontend::ClaudeCode => claude::bundle(files, config, root_command)?,
    }
    Ok(())
}
