use super::super::{Config, Files, bundle, template};
use super::wizard;
use crate::project_config;
use anyhow::{Result, ensure};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub fn run(root: &Path, args: &[String]) -> Result<i32> {
    let interactive = args == ["--interactive"];
    if interactive {
        return wizard::run(root);
    }
    let options = template::options(root, args)?;
    let config = template::config(&options);
    let files = bundle(&config)?;
    prepare(root, &config, &files)?;
    install(root, &config, &files)?;
    println!(
        "Initialized {} scaffold with runtime {}. Run config-check and doctor; source files remain yours to create.",
        options["language"],
        project_config::VERSION
    );
    Ok(0)
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
    let root = preview.path().canonicalize()?;
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
pub(super) fn install(root: &Path, config: &Config, files: &Files) -> Result<()> {
    for (path, contents) in files {
        let executable = executable(path);
        let path = project_config::relative(root, path)?;
        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("install path has no parent"))?;
        fs::create_dir_all(parent)?;
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
    crate::shared::vcs::initialize(root, &config.git.base)?;
    fs::create_dir_all(project_config::relative(root, &config.paths.runtime)?)?;
    crate::shared::vcs::Repository::new(root).register_hooks(&config.paths.service_path("hooks"))
}
fn executable(path: &str) -> bool {
    Path::new(path)
        .components()
        .any(|component| component.as_os_str() == "bin" || component.as_os_str() == "hooks")
}
