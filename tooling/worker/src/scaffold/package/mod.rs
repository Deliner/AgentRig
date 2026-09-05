mod adapters;
mod assets;
mod doctor;
mod lint;
pub(super) mod manifest;
mod template;
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
    let options = template::options(root, args)?;
    let config = template::config(&options);
    let files = bundle(&options, &config)?;
    check_collisions(root, &files)?;
    // Check existing Git hook ownership before creating any files.
    let git = root.join(".git").exists();
    if git {
        let existing =
            crate::util::git(root, &["config", "--get", "core.hooksPath"]).unwrap_or_default();
        ensure!(
            existing.is_empty(),
            "existing core.hooksPath={existing}; init will not replace it"
        );
    }
    validate_bundle(&files)?;
    install(root, &files)?;
    if git {
        crate::util::git(root, &["config", "core.hooksPath", ".worker/hooks"])?;
    }
    println!(
        "Initialized {} scaffold with runtime {}. Run config-check and doctor; source files remain yours to create.",
        options["language"],
        config::VERSION
    );
    Ok(0)
}
fn bundle(options: &template::Options<'_>, config: &Config) -> Result<Files> {
    let skill_root = options["skills"];
    let mut files = BTreeMap::<String, Vec<u8>>::new();
    files.insert(
        config::FILE.into(),
        toml::to_string_pretty(config)?.into_bytes(),
    );
    for (name, source) in assets::skills() {
        files.insert(format!("{skill_root}/{name}/SKILL.md"), source.into_bytes());
    }
    for (name, source) in assets::memory() {
        files.insert(
            format!("{}/{name}.md", options["memory"]),
            source.into_bytes(),
        );
    }
    files.insert(
        ".worker/lint.toml".into(),
        lint::template(skill_root, options["source"])?.into_bytes(),
    );
    files.insert(
        ".worker/reminder.json".into(),
        include_bytes!("../../../assets/skills/complexity-discipline/context-reminder.json")
            .to_vec(),
    );
    add_runtime(&mut files)?;
    files.insert(manifest::PATH.into(), manifest::installed(&files, config)?);
    Ok(files)
}
fn add_runtime(files: &mut Files) -> Result<()> {
    files.insert(".worker/.gitignore".into(), b"runtime/\n".to_vec());
    files.insert(
        ".worker/bin/discipline-worker".into(),
        fs::read(std::env::current_exe()?)?,
    );
    files.insert("justfile".into(), template::justfile().into_bytes());
    files.insert(
        ".codex/config.toml".into(),
        adapters::CODEX_CONFIG.as_bytes().to_vec(),
    );
    files.insert(".codex/hooks.json".into(), adapters::registration()?);
    for (path, contents) in adapters::git_hooks() {
        files.insert(path.into(), contents.to_vec());
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
