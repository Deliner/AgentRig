use super::super::{
    config::{Config, FILE},
    package::manifest::{self, Manifest},
};
use anyhow::{Context as _, Result, ensure};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

pub const FROM: &str = "0.1.0";
pub const TO: &str = "0.2.0";
pub fn configuration(root: &Path) -> Result<Config> {
    Ok(toml::from_str(&fs::read_to_string(root.join(FILE))?)?)
}
pub fn version(binary: &Path) -> Result<String> {
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .context("execute release version probe")?;
    ensure!(output.status.success(), "release version probe failed");
    let text = String::from_utf8(output.stdout)?;
    Ok(text
        .trim()
        .strip_prefix("discipline-worker ")
        .context("not a worker release")?
        .into())
}
pub fn export(binary: &Path, config: &Config) -> Result<tempfile::TempDir> {
    let directory = tempfile::tempdir()?;
    let mut command = Command::new(binary);
    let review = config.capabilities.review.is_some();
    if review {
        command.args(["init", "--review", "true"]);
    } else {
        command.arg("init");
    }
    let output = command
        .arg("--root")
        .arg(directory.path())
        .args([
            "--skills",
            &config.paths.skills,
            "--memory",
            &config.paths.memory,
            "--base",
            &config.git.base,
            "--prefix",
            &config.git.prefix,
        ])
        .output()?;
    ensure!(
        output.status.success(),
        "cannot export release: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(directory)
}
pub fn manifest(root: &Path) -> Result<Manifest> {
    let path = root.join(manifest::PATH);
    let present = path.is_file();
    if present {
        let result: Manifest = serde_json::from_slice(&fs::read(path)?)?;
        ensure!(
            result.manifest_version == 1 && result.config_schema == 1,
            "unsupported installation manifest schema"
        );
        return Ok(result);
    }
    let mut files = BTreeMap::new();
    collect(root, Path::new(""), &mut files)?;
    Ok(serde_json::from_slice(&manifest::installed(
        &files,
        &configuration(root)?,
    )?)?)
}
fn collect(root: &Path, relative: &Path, files: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
    for entry in fs::read_dir(root.join(relative))? {
        let entry = entry?;
        let path = relative.join(entry.file_name());
        let directory = entry.file_type()?.is_dir();
        ensure!(
            !entry.file_type()?.is_symlink(),
            "release contains a symlink: {}",
            path.display()
        );
        if directory {
            collect(root, &path, files)?;
        } else {
            files.insert(path.to_string_lossy().into_owned(), fs::read(entry.path())?);
        }
    }
    Ok(())
}
pub fn migrated(source: &str) -> Result<Vec<u8>> {
    #[derive(serde::Deserialize)]
    struct Pin {
        runtime: toml::Spanned<String>,
        paths: Policy,
    }
    #[derive(serde::Deserialize)]
    struct Policy {
        lint: toml::Spanned<String>,
    }
    let pin: Pin = toml::from_str(source)?;
    ensure!(
        pin.runtime.get_ref() == FROM,
        "only {FROM} -> {TO} is implemented"
    );
    let mut updated = source.to_owned();
    let mut replacements = vec![
        (pin.runtime.span(), format!("\"{TO}\"")),
        (
            pin.paths.lint.span(),
            serde_json::to_string(&lint_path(pin.paths.lint.get_ref()))?,
        ),
    ];
    replacements.sort_by_key(|(span, _)| std::cmp::Reverse(span.start));
    for (span, value) in replacements {
        updated.replace_range(span, &value);
    }
    Ok(updated.into_bytes())
}

pub fn lint_path(path: &str) -> String {
    Path::new(path)
        .with_extension("yaml")
        .to_string_lossy()
        .into_owned()
}

pub fn lint_yaml(root: &Path, path: &str) -> Result<Vec<u8>> {
    let source = fs::read_to_string(root.join(path))?;
    let value: toml::Value = toml::from_str(&source)?;
    Ok(review_runner::config::yaml::encode(&value)?.into_bytes())
}
