use super::super::{Files, bundle, config, manifest, template};
use super::wizard;
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
    reject_legacy(root)?;
    let options = template::options(root, args)?;
    let config = template::config(&options);
    let files = bundle(root, &config)?;
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
pub(super) fn reject_legacy(root: &Path) -> Result<()> {
    ensure!(
        !root.join(crate::scaffold::recovery::LEGACY_FILE).exists(),
        "legacy installation requires explicit upgrade; existing configuration preserved"
    );
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
