use super::{
    commands,
    config::{self, CheckKind, Context},
};
use crate::lint::{self, config::globs, inventory};
use anyhow::{Result, ensure};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn run(root: &Path, staged: bool) -> Result<i32> {
    let snapshot = if staged { Some(export(root)?) } else { None };
    let tree = snapshot.as_ref().map(|dir| dir.path()).unwrap_or(root);
    let context = Context::load(tree)?;
    let files = files(&context)?;
    for check in &context.config.checks {
        let include = globs(&check.include)?;
        let applies = files.iter().any(|path| include.is_match(path));
        if !applies {
            println!("SKIP [{}]: no selected files", check.id);
            continue;
        }
        let result = match check.kind {
            CheckKind::Command => commands::run(
                &context,
                check.command.as_deref().expect("validated command"),
                &[],
            ),
            CheckKind::Lint => lint::run(
                tree,
                &context.path(&context.config.paths.lint)?,
                false,
                false,
            ),
            CheckKind::Memory => super::memory::check_with_history(&context, root),
        };
        let code = match result {
            Ok(code) => code,
            Err(error) => {
                eprintln!(
                    "ERROR [{}]: {error:#}. ACTION: Apply {}",
                    check.id, check.skill
                );
                return Ok(2);
            }
        };
        if code == 0 {
            println!("PASS [{}]", check.id);
            continue;
        }
        let level = if check.warning { "WARNING" } else { "ERROR" };
        eprintln!(
            "{level} [{}]: exited {code}. ACTION: Apply {}",
            check.id, check.skill
        );
        let interrupted = code >= 128;
        if interrupted || !check.warning {
            return Ok(code);
        }
    }
    Ok(0)
}
fn export(root: &Path) -> Result<tempfile::TempDir> {
    let directory = tempfile::tempdir()?;
    let prefix = format!("--prefix={}/", directory.path().display());
    let result = Command::new("git")
        .args(["checkout-index", "--all", &prefix])
        .current_dir(root)
        .output()?;
    ensure!(
        result.status.success(),
        "cannot export index: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    ensure!(
        directory.path().join(config::FILE).is_file(),
        "stage worker.toml before checking the index"
    );
    Ok(directory)
}
pub fn files(context: &Context) -> Result<Vec<PathBuf>> {
    let excludes = vec![
        ".git".into(),
        ".git/**".into(),
        context.config.paths.runtime.clone(),
        format!("{}/**", context.config.paths.runtime),
    ];
    Ok(inventory::collect(&context.root, &globs(&excludes)?)?.files)
}
