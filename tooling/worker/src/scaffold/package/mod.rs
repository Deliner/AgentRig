pub(super) mod adapters;
mod assets;
mod doctor;
mod lint;
pub(super) mod manifest;
mod review;
pub(super) mod setup;
pub use setup::run as setup;
mod template;
mod wizard;
use super::config::{self, Config};
use anyhow::{Result, ensure};
pub use doctor::run as doctor;
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
type Files = BTreeMap<String, Vec<u8>>;
pub fn init(root: &Path, args: &[String]) -> Result<i32> {
    let interactive = args == ["--interactive"];
    if interactive {
        return wizard::run(root);
    }
    reject_legacy(root)?;
    let options = template::options(root, args)?;
    let config = template::config(&options);
    let files = bundle(&config)?;
    check_collisions(root, &files)?;
    // Check native hook ownership before creating any files.
    let repository = config.vcs.backend.repository(root)?;
    if let Some(repository) = &repository {
        repository.validate_registration(&config.paths.service_path("hooks"))?;
        let existing = repository.hook_registration()?;
        ensure!(
            existing.is_empty(),
            "existing VCS hook registration={existing}; init will not replace it"
        );
    }
    validate_bundle(&files)?;
    install(root, &files)?;
    if let Some(repository) = repository {
        repository.register_hooks(&config.paths.service_path("hooks"))?;
    }
    println!(
        "Initialized {} scaffold with runtime {}. Run config-check and doctor; source files remain yours to create.",
        options["language"],
        config::VERSION
    );
    Ok(0)
}
fn reject_legacy(root: &Path) -> Result<()> {
    ensure!(
        !root.join(super::upgrade::migration::LEGACY_FILE).exists(),
        "legacy installation requires explicit upgrade; existing configuration preserved"
    );
    Ok(())
}
fn bundle(config: &Config) -> Result<Files> {
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
    add_runtime(&mut files, config)?;
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
fn add_runtime(files: &mut Files, config: &Config) -> Result<()> {
    let mercurial = config.vcs.backend == review_runner::vcs::Kind::Mercurial.into();
    if mercurial {
        files.insert(
            config.paths.service_path("hooks.hgignore"),
            format!(
                "syntax: glob\n{}/**\n{}/review/runtime/**\n{}/review/reports/**\n",
                config.paths.runtime, config.paths.service, config.paths.service
            )
            .into_bytes(),
        );
    }
    files.insert(
        config.paths.service_path(".gitignore"),
        b"runtime/\n/inputs/runtime/\n/inputs/reports/\n".to_vec(),
    );
    files.insert(
        config.paths.service_path("bin/agentrig"),
        fs::read(std::env::current_exe()?)?,
    );
    files.insert("justfile".into(), template::justfile(config).into_bytes());
    files.insert(
        ".codex/config.toml".into(),
        adapters::CODEX_CONFIG.as_bytes().to_vec(),
    );
    files.insert(".codex/hooks.json".into(), adapters::registration(config)?);
    for (path, contents) in adapters::vcs_hooks(config)? {
        files.insert(path, contents);
    }
    Ok(())
}
fn check_collisions(root: &Path, files: &Files) -> Result<()> {
    for path in files.keys() {
        let resolved = config::relative(root, path)?;
        for parent in resolved
            .ancestors()
            .skip(1)
            .take_while(|p| p.starts_with(root))
        {
            ensure!(
                !parent.exists() || parent.is_dir(),
                "init collision: {} is not a directory",
                parent.display()
            );
        }
        ensure!(
            !resolved.exists() && root.join(path).symlink_metadata().is_err(),
            "init collision: {path}; existing files were preserved"
        );
    }
    Ok(())
}
fn validate_bundle(files: &Files) -> Result<()> {
    // Validate generated files through the same loader before touching the consumer.
    // This also detects a generated file being another generated file's parent.
    let preview = tempfile::tempdir()?;
    for (path, contents) in files {
        let path = config::relative(preview.path(), path)?;
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, contents)?;
    }
    config::Context::load(preview.path())?;
    Ok(())
}
fn install(root: &Path, files: &Files) -> Result<()> {
    for (path, contents) in files {
        let executable = manifest::executable(path);
        let path = config::relative(root, path)?;
        fs::create_dir_all(path.parent().unwrap())?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(contents)?;
        if executable {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        }
    }
    Ok(())
}
