use super::wizard;
use crate::project_config;
use crate::project_setup::template::{self, Scaffold};
use crate::project_setup::{Config, Files, bundle};
use crate::shared::console;
use crate::shared::fs::{self, OpenOptions, Permissions, PermissionsExt};
use anyhow::{Result, ensure};
use std::{io::Write, path::Path};

pub fn run(root: &Path, args: &[String]) -> Result<i32> {
    let interactive = args == ["--interactive"];
    if interactive {
        return wizard::run(root);
    }
    let options = template::options(root, args)?;
    let scaffold = template::scaffold(root, &options)?;
    let files = bundle(&scaffold)?;
    prepare(root, &scaffold.config, &files)?;
    install(root, &scaffold, &files)?;
    console::stdout(format!(
        "Initialized {} scaffold with runtime {}. Run config-check and doctor; source files remain yours to create.",
        options["language"],
        project_config::VERSION
    ));
    offer_ratchet(root, &scaffold.config)?;
    Ok(0)
}
// DECISION: D123
pub(super) fn offer_ratchet(root: &Path, config: &Config) -> Result<()> {
    let disabled = !config.capabilities.lint;
    if disabled {
        return Ok(());
    }
    let lint = project_config::relative(root, &config.paths.lint)?;
    let errors = crate::structural_lint::baseline::errors(root, &lint)?;
    let existing = errors > 0;
    if existing {
        console::stdout(format!(
            "Lint errors in existing files: {errors}. To adopt the rules without blocking the gate, run just lint-baseline: lint then fails only when a rule's errors in a path grow."
        ));
    }
    Ok(())
}
pub(super) fn prepare(root: &Path, config: &Config, files: &Files) -> Result<()> {
    check_collisions(root, files)?;
    let repository = crate::shared::vcs::Repository::discover(root)?;
    if let Some(repository) = &repository {
        repository.validate_registration(&config.paths.service_path("hooks"))?;
        let existing = repository.hook_registration()?;
        ensure!(
            existing.is_empty(),
            "existing VCS hook registration={existing}; init will not replace it"
        );
    }
    validate_bundle(files)
}
fn check_collisions(root: &Path, files: &Files) -> Result<()> {
    for path in files.keys() {
        let resolved = project_config::relative(root, path)?;
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
    let preview = tempfile::tempdir()?;
    let root = fs::canonicalize(preview.path())?;
    for (path, contents) in files {
        let path = project_config::relative(&root, path)?;
        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("bundle path has no parent"))?;
        fs::create_dir_all(parent)?;
        fs::write(path, contents)?;
    }
    project_config::Context::load(&root)?;
    Ok(())
}
pub(super) fn install(root: &Path, scaffold: &Scaffold, files: &Files) -> Result<()> {
    let config = &scaffold.config;
    for (path, contents) in files {
        let executable = executable(path);
        let path = project_config::relative(root, path)?;
        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("install path has no parent"))?;
        fs::create_dir_all(parent)?;
        let mut file = fs::open_with(OpenOptions::new().write(true).create_new(true), &path)?;
        file.write_all(contents)?;
        if executable {
            fs::set_permissions(&path, Permissions::from_mode(0o755))?;
        }
    }
    if let Some(settings) = &scaffold.clippy {
        settings.write()?;
    }
    crate::shared::vcs::initialize(root, &config.git.base)?;
    fs::create_dir_all(project_config::relative(root, &config.paths.runtime)?)?;
    crate::shared::vcs::Repository::new(root).register_hooks(&config.paths.service_path("hooks"))
}
fn executable(path: &str) -> bool {
    Path::new(path)
        .components()
        .any(|component| component.as_os_str() == "bin" || component.as_os_str() == "hooks")
}
