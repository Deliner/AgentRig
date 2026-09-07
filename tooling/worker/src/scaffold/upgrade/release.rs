use super::super::{
    config::Config,
    package::manifest::{self, Manifest},
};
use anyhow::{Context as _, Result, ensure};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

pub const FROM: &str = "0.2.0";
pub const TO: &str = "0.3.0";
pub fn version(binary: &Path) -> Result<String> {
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .context("execute release version probe")?;
    ensure!(output.status.success(), "release version probe failed");
    let text = String::from_utf8(output.stdout)?;
    Ok(text
        .trim()
        .strip_prefix("agentrig ")
        .or_else(|| {
            text.trim()
                .strip_prefix("discipline-worker ")
                .filter(|version| *version == FROM)
        })
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
    let target = version(binary)? == TO;
    if target {
        command.args(["--service", &config.paths.service]);
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
            &config.vcs.base,
            "--prefix",
            &config.vcs.prefix,
        ])
        .output()?;
    ensure!(
        output.status.success(),
        "cannot export release: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    if target {
        selected_guidance(directory.path(), config)?;
    }
    Ok(directory)
}

fn selected_guidance(root: &Path, config: &Config) -> Result<()> {
    let files = crate::scaffold::package::guidance(config);
    let selected: Manifest = serde_json::from_slice(&manifest::installed(&files, config)?)?;
    let receipt = root.join(config.paths.service_path("manifest.json"));
    let mut current: Manifest = serde_json::from_slice(&fs::read(&receipt)?)?;
    for (name, bytes) in files {
        let path = crate::scaffold::config::relative(root, &name)?;
        fs::create_dir_all(path.parent().context("guidance parent required")?)?;
        fs::write(path, bytes)?;
    }
    current.files.extend(selected.files);
    fs::write(receipt, serde_json::to_vec_pretty(&current)?)?;
    Ok(())
}
pub fn manifest(root: &Path) -> Result<Manifest> {
    let path = root.join(super::migration::MANIFEST);
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
        &super::migration::configuration(root)?,
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
    let mut config: toml::Value = toml::from_str(source)?;
    ensure!(
        config.get("runtime").and_then(toml::Value::as_str) == Some(FROM),
        "only {FROM} -> {TO} is implemented"
    );
    let lint = config["paths"]["lint"]
        .as_str()
        .context("paths.lint required")?;
    let target = lint_path(lint);
    config["runtime"] = toml::Value::String(TO.into());
    config["paths"]["lint"] = toml::Value::String(target);
    config["paths"]
        .as_table_mut()
        .context("paths table required")?
        .insert("service".into(), toml::Value::String(".worker".into()));
    for capability in ["review", "delegation"] {
        let reference = config
            .get_mut("capabilities")
            .and_then(|caps| caps.get_mut(capability))
            .and_then(|entry| entry.get_mut("config"));
        if let Some(path) = reference {
            *path = toml::Value::String(lint_path(
                path.as_str().context("capability config path required")?,
            ));
        }
    }
    Ok(review_runner::config::yaml::encode(&config)?.into_bytes())
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
