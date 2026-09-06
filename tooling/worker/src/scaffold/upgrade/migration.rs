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

pub struct Resources {
    pub converted: BTreeMap<String, Vec<u8>>,
    pub imported: agentrig::resources::Bundle,
}

pub fn resources(root: &Path, config: &Config) -> Result<Resources> {
    let mut files = BTreeMap::new();
    let mut imported = agentrig::resources::Bundle::new(&config.paths.service);
    if let Some(review) = &config.capabilities.review {
        review_resources(root, &review.config, &mut files, &mut imported)?;
    }
    if let Some(delegate) = &config.capabilities.delegation {
        files.insert(
            delegate.config.clone(),
            super::release::lint_yaml(root, &delegate.config)?,
        );
    }
    Ok(Resources {
        converted: files,
        imported,
    })
}

fn review_resources(
    root: &Path,
    path: &str,
    files: &mut BTreeMap<String, Vec<u8>>,
    imported: &mut agentrig::resources::Bundle,
) -> Result<()> {
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
        let target = match absolute.strip_prefix(root) {
            Ok(relative) => {
                let name = relative
                    .to_str()
                    .context("project config path must be UTF-8")?;
                files.insert(name.into(), super::release::lint_yaml(root, name)?);
                super::release::lint_path(project)
            }
            Err(_) => {
                let target = import_project(&absolute, imported)?;
                from_config(root, path, &target)?
            }
        };
        *reference = toml::Value::String(target);
    }
    files.insert(
        path.into(),
        review_runner::config::yaml::encode(&value)?.into_bytes(),
    );
    Ok(())
}

fn import_project(path: &Path, imported: &mut agentrig::resources::Bundle) -> Result<String> {
    let mut value: toml::Value = toml::from_str(std::str::from_utf8(&imported.read(path)?)?)?;
    let contract = value
        .get_mut("review")
        .and_then(|review| review.get_mut("contract"))
        .context("review.contract required")?;
    let source = path
        .parent()
        .unwrap()
        .join(contract.as_str().context("contract path required")?);
    let copied = imported.copy(&source)?;
    *contract = toml::Value::String(imported.sibling(&copied)?.to_string_lossy().into_owned());
    let name = path.with_extension("yaml");
    imported.put(
        name.file_name()
            .and_then(|name| name.to_str())
            .context("project config name must be UTF-8")?,
        review_runner::config::yaml::encode(&value)?.into_bytes(),
        false,
    )
}

fn from_config(root: &Path, config: &str, target: &str) -> Result<String> {
    let config = crate::util::resolve(&root.join(config))?;
    let parent = config.parent().context("configuration parent required")?;
    let depth = parent.strip_prefix(root)?.components().count();
    let prefix: std::path::PathBuf = std::iter::repeat_n("..", depth).collect();
    Ok(prefix.join(target).to_string_lossy().into_owned())
}

impl Resources {
    pub fn register(&self, manifest: &mut crate::scaffold::package::manifest::Manifest) {
        use crate::scaffold::package::manifest::{Entry, Ownership, checksum};
        for (path, bytes) in &self.converted {
            manifest.files.insert(
                super::release::lint_path(path),
                Entry {
                    sha256: checksum(bytes),
                    ownership: Ownership::Configuration,
                    executable: false,
                },
            );
        }
        for (path, file) in &self.imported.files {
            manifest.files.insert(
                path.clone(),
                Entry {
                    sha256: checksum(&file.bytes),
                    ownership: Ownership::Asset,
                    executable: file.executable,
                },
            );
        }
    }
}
