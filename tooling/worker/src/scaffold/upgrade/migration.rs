use crate::scaffold::config::{self, Config};
use crate::scaffold::package::manifest::{Entry, Ownership, checksum};
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
            delegate_resources(root, &delegate.config, &mut imported)?,
        );
    }
    Ok(Resources {
        converted: files,
        imported,
    })
}

fn delegate_resources(
    root: &Path,
    path: &str,
    imported: &mut agentrig::resources::Bundle,
) -> Result<Vec<u8>> {
    let source = toml::from_str(&fs::read_to_string(root.join(path))?)?;
    let mut config = agentrig::delegate::config::resolve(&root.join(path), source)?;
    for profile in config.profiles.values_mut() {
        profile.prompt = relocate(root, path, &profile.prompt, imported)?.into();
        for skill in &mut profile.environment.skills {
            *skill = relocate(root, path, skill, imported)?.into();
        }
        for program in profile.environment.programs.values_mut() {
            *program = relocate(root, path, program, imported)?.into();
        }
    }
    Ok(review_runner::config::yaml::encode(&config)?.into_bytes())
}

fn relocate(
    root: &Path,
    config: &str,
    resource: &Path,
    imported: &mut agentrig::resources::Bundle,
) -> Result<String> {
    let target = match resource.strip_prefix(root) {
        Ok(path) => path
            .to_str()
            .context("resource path must be UTF-8")?
            .to_owned(),
        Err(_) => {
            let directory = resource.is_dir();
            if directory {
                imported.directory(resource)?
            } else {
                imported.copy(resource)?
            }
        }
    };
    from_config(root, config, &target)
}

fn review_resources(
    root: &Path,
    path: &str,
    files: &mut BTreeMap<String, Vec<u8>>,
    imported: &mut agentrig::resources::Bundle,
) -> Result<()> {
    let mut value: toml::Value = toml::from_str(&fs::read_to_string(root.join(path))?)?;
    review_prompts(root, path, &mut value, imported)?;
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
        let absolute = agentrig::paths::resolve(&root.join(path).parent().unwrap().join(project))?;
        let target = match absolute.strip_prefix(root) {
            Ok(relative) => {
                let name = relative
                    .to_str()
                    .context("project config path must be UTF-8")?;
                files.insert(name.into(), project_yaml(root, name, imported)?);
                super::release::lint_path(project)
            }
            Err(_) => {
                let target = import_project(&absolute, imported)?;
                from_config(root, path, &target)?
            }
        };
        *reference = toml::Value::String(target);
    }
    let encoded = review_runner::config::yaml::encode(&value)?;
    files.insert(path.into(), encoded.into_bytes());
    Ok(())
}

fn review_prompts(
    root: &Path,
    path: &str,
    value: &mut toml::Value,
    imported: &mut agentrig::resources::Bundle,
) -> Result<()> {
    let reviewers = value
        .get_mut("reviewers")
        .and_then(toml::Value::as_table_mut)
        .context("legacy reviewers required")?;
    for (_, reviewer) in reviewers.iter_mut() {
        portable_reference(
            root,
            path,
            reviewer
                .get_mut("prompt")
                .context("reviewer prompt required")?,
            imported,
        )?;
    }
    Ok(())
}

fn project_yaml(
    root: &Path,
    path: &str,
    imported: &mut agentrig::resources::Bundle,
) -> Result<Vec<u8>> {
    let mut value: toml::Value = toml::from_str(&fs::read_to_string(root.join(path))?)?;
    let contract = value
        .get_mut("review")
        .and_then(|review| review.get_mut("contract"))
        .context("review.contract required")?;
    portable_reference(root, path, contract, imported)?;
    Ok(review_runner::config::yaml::encode(&value)?.into_bytes())
}

fn portable_reference(
    root: &Path,
    config: &str,
    value: &mut toml::Value,
    imported: &mut agentrig::resources::Bundle,
) -> Result<()> {
    let resource = review_runner::config::resource(
        &root.join(config),
        Path::new(value.as_str().context("resource path required")?),
    )?;
    let external = !resource.starts_with(root);
    if external {
        *value = toml::Value::String(relocate(root, config, &resource, imported)?);
    }
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
    let config = agentrig::paths::resolve(&root.join(config))?;
    let parent = config.parent().context("configuration parent required")?;
    let depth = parent.strip_prefix(root)?.components().count();
    let prefix: std::path::PathBuf = std::iter::repeat_n("..", depth).collect();
    Ok(prefix.join(target).to_string_lossy().into_owned())
}

impl Resources {
    pub fn register(&self, manifest: &mut crate::scaffold::package::manifest::Manifest) {
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
