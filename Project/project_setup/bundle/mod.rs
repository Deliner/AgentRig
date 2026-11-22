mod adapters;
mod assets;
mod claude;
mod contents;
mod lint;
mod recipes;
mod runtime;
mod skills;
mod template;
use crate::project_config::{self, Config};
use crate::project_setup::rust::Version;
use crate::shared::calling_agent::Agent;
pub use adapters::{hooks_registered, vcs_hooks};
use anyhow::{Result, ensure};
pub use claude::registration as claude_registration;
pub use contents::{Files, skill_links};
pub use recipes::{Recipe, recipe};
pub use runtime::add as runtime_add;
pub use template::{Scaffold, options, scaffold};

pub fn files(scaffold: &Scaffold) -> Result<Files> {
    let config = &scaffold.config;
    let mut files = guidance(config);
    let rust_project = scaffold.clippy.is_some();
    if rust_project {
        add_rust_capabilities(&mut files)?;
    }
    for file in project_config::files(config)? {
        files.insert(file.path, file.source.into_bytes());
    }
    for (name, source) in assets::memory()? {
        files.insert(
            format!("{}/{name}.md", config.paths.memory),
            source.into_bytes(),
        );
    }
    add_policy(&mut files, scaffold)?;
    add_runtime(&mut files, config)?;
    Ok(files)
}

fn guidance(config: &Config) -> Files {
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
// DECISION: D175
fn add_rust_capabilities(files: &mut Files) -> Result<()> {
    let capabilities = Version::rig()?.capabilities();
    let instructions = files.entry("AGENTS.md".into()).or_default();
    instructions.push(b'\n');
    instructions.extend_from_slice(capabilities.as_bytes());
    Ok(())
}
fn add_policy(files: &mut Files, scaffold: &Scaffold) -> Result<()> {
    let config = &scaffold.config;
    // DECISION: D198
    let rust_project = scaffold.clippy.is_some();
    let language = if rust_project { "rust" } else { "python" };
    if config.capabilities.lint {
        files.insert(
            config.paths.lint.clone(),
            lint::template(config, &scaffold.rust, language)?.into_bytes(),
        );
    }
    if let Some(path) = &config.hooks.reminder {
        files.insert(
            path.clone(),
            include_bytes!("../../shared/skills/complexity-discipline/context-reminder.json")
                .to_vec(),
        );
    }
    Ok(())
}
fn add_runtime(files: &mut Files, config: &Config) -> Result<()> {
    let generated = adapters::generated(config)?;
    // DECISION: D189
    files.insert(format!("{}/.gitignore", template::CACHE), b"*\n".to_vec());
    files.extend(runtime::files(config)?);
    for justfile in recipes::justfiles(config) {
        files.insert(justfile.path, justfile.source.into_bytes());
    }
    client_files(files, config)?;
    for (path, contents) in generated {
        ensure!(
            !files.contains_key(&path),
            "generated VCS file conflicts with another resource: {path}"
        );
        files.insert(path, contents.into_bytes());
    }
    Ok(())
}
// DECISION: D188
fn client_files(files: &mut Files, config: &Config) -> Result<()> {
    files.insert(
        ".codex/config.toml".into(),
        adapters::CODEX_CONFIG.as_bytes().to_vec(),
    );
    files.insert(
        ".codex/hooks.json".into(),
        adapters::registration(config, Agent::Codex)?,
    );
    claude::bundle(files, config)
}
