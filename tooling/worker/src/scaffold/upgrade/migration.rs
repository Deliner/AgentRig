use crate::scaffold::config::{self, Config};
use anyhow::{Context as _, Result};
use std::{collections::BTreeMap, fs, path::Path};

pub const LEGACY_FILE: &str = "worker.toml";
pub const MANIFEST: &str = ".worker/manifest.json";

// Legacy parsing belongs to explicit conversion and historical/recovery metadata,
// never to the runtime configuration loader.
pub fn configuration(root: &Path) -> Result<Config> {
    let mut config: Config = toml::from_str(&fs::read_to_string(root.join(LEGACY_FILE))?)?;
    config.paths.service = ".worker".into();
    Ok(config)
}

pub fn recovery_runtime(root: &Path) -> Result<String> {
    let legacy = root.join(LEGACY_FILE).is_file();
    if legacy {
        let source = fs::read_to_string(root.join(LEGACY_FILE))?;
        let value: toml::Value = toml::from_str(&source)?;
        Ok(value
            .get("paths")
            .and_then(|paths| paths.get("runtime"))
            .and_then(toml::Value::as_str)
            .context("paths.runtime required for upgrade recovery")?
            .into())
    } else {
        Ok(config::read(root)?.paths.runtime)
    }
}

pub fn historical_memory(source: &str) -> Result<String> {
    let value: toml::Value = toml::from_str(source)?;
    Ok(value
        .get("paths")
        .and_then(|paths| paths.get("memory"))
        .and_then(toml::Value::as_str)
        .context("committed legacy paths.memory required")?
        .into())
}

pub fn resources(root: &Path, config: &Config) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    if let Some(review) = &config.capabilities.review {
        review_resources(root, &review.config, &mut files)?;
    }
    if let Some(delegate) = &config.capabilities.delegation {
        files.insert(
            delegate.config.clone(),
            super::release::lint_yaml(root, &delegate.config)?,
        );
    }
    Ok(files)
}

fn review_resources(root: &Path, path: &str, files: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
    let mut value: toml::Value = toml::from_str(&fs::read_to_string(root.join(path))?)?;
    let tools = value
        .get_mut("tools")
        .and_then(toml::Value::as_table_mut)
        .context("legacy review tools required")?;
    for (_, tool) in tools.iter_mut() {
        let reference = tool
            .get_mut("project_config")
            .context("project_config required")?;
        let project = reference
            .as_str()
            .context("project_config must be a path")?;
        let absolute = crate::util::resolve(&root.join(path).parent().unwrap().join(project))?;
        let relative = absolute.strip_prefix(root).context(
            "explicit migration requires review project configs inside the installation",
        )?;
        let name = relative
            .to_str()
            .context("project config path must be UTF-8")?;
        files.insert(name.into(), super::release::lint_yaml(root, name)?);
        *reference = toml::Value::String(super::release::lint_path(project));
    }
    files.insert(
        path.into(),
        review_runner::config::yaml::encode(&value)?.into_bytes(),
    );
    Ok(())
}
