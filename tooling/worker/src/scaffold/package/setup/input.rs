mod composed;
mod delegation;
mod lint;
mod review;

use super::{Config, Files, config, manifest};
use crate::{composition, resources};
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Prepared {
    pub config: Config,
    pub files: Files,
    source: Option<Source>,
}

struct Source {
    resolved: composition::Resolved,
    resources: resources::Bundle,
    stock: Files,
    configurations: std::collections::BTreeMap<PathBuf, composition::Resolved>,
}

pub fn prepare(root: &Path, path: Option<&Path>) -> Result<Prepared> {
    match path {
        Some(path) => {
            let prepared = external(root, path)?;
            unchanged(root, &prepared.config, &prepared.files)?;
            Ok(prepared)
        }
        None => {
            let config = config::read(root)?;
            let files = super::super::bundle(root, &config)?;
            Ok(Prepared {
                config,
                files,
                source: None,
            })
        }
    }
}

pub(super) fn external(root: &Path, path: &Path) -> Result<Prepared> {
    let resolved = composition::resolve(path)?;
    let mut config: Config = review_runner::config::yaml::decode(
        &review_runner::config::yaml::encode(&resolved.configuration)?,
    )?;
    let mut source = Source {
        resources: resources::Bundle::new(&config.paths.service),
        stock: super::super::bundle(root, &config)?,
        resolved,
        configurations: Default::default(),
    };
    source.guidance(&mut config)?;
    lint::prepare(&mut source, &mut config)?;
    review::prepare(&mut source, &mut config)?;
    delegation::prepare(&mut source, &mut config)?;
    source.environment(&mut config)?;
    source.validate_packages()?;
    let files = source.files(&config)?;
    Ok(Prepared {
        config,
        files,
        source: Some(source),
    })
}

impl Prepared {
    pub fn verify(&self) -> Result<()> {
        let Some(source) = &self.source else {
            return Ok(());
        };
        source.resources.verify()?;
        for (path, expected) in
            std::iter::once((&source.resolved.root, &source.resolved.root_digest)).chain(
                source
                    .resolved
                    .packages
                    .iter()
                    .map(|package| (&package.path, &package.digest)),
            )
        {
            ensure!(
                resources::digest(&fs::read(path)?) == *expected,
                "configuration changed before installation: {}",
                path.display()
            );
        }
        Ok(())
    }
}

impl Source {
    fn path(&self, address: &str, value: &str) -> Result<Option<PathBuf>> {
        let declaring = self.resolved.origin(address);
        let path = declaring
            .parent()
            .context("configuration directory required")?
            .join(value);
        let shipped = !path.exists() && self.stock.contains_key(value);
        if shipped {
            return Ok(None);
        }
        Ok(Some(path))
    }

    fn skill(&mut self, value: &mut String, address: &str) -> Result<()> {
        let Some(path) = self.path(address, value)? else {
            return Ok(());
        };
        *value = self.materialize_skill(&path, value)?;
        Ok(())
    }

    fn materialize_skill(&mut self, path: &Path, value: &str) -> Result<String> {
        let shipped = self.stock.contains_key(value)
            && path.file_name().is_some_and(|name| name == "SKILL.md");
        if shipped {
            let target = Path::new(value)
                .parent()
                .and_then(|path| path.to_str())
                .context("skill target directory required")?;
            self.resources.directory_at(
                path.parent().context("skill source directory required")?,
                target,
            )?;
            return Ok(value.into());
        }
        self.skill_path(path)
    }

    fn skill_path(&mut self, path: &Path) -> Result<String> {
        let skill = path.file_name().is_some_and(|name| name == "SKILL.md");
        if skill {
            let directory = path.parent().context("skill directory required")?;
            Ok(format!("{}/SKILL.md", self.resources.directory(directory)?))
        } else {
            self.resources.copy(path)
        }
    }

    fn guidance(&mut self, config: &mut Config) -> Result<()> {
        self.skill(&mut config.config_skill, "/config_skill")?;
        for check in &mut config.checks {
            self.skill(&mut check.skill, &format!("/checks/{}/skill", check.id))?;
        }
        for route in &mut config.hooks.routes {
            self.skill(&mut route.skill, "/hooks/routes")?;
        }
        if let Some(skill) = &mut config.hooks.discipline_skill {
            self.skill(skill, "/hooks/discipline_skill")?;
        }
        if let Some(reminder) = &mut config.hooks.reminder {
            let Some(path) = self.path("/hooks/reminder", reminder)? else {
                return Ok(());
            };
            self.stock.remove(reminder);
            *reminder = self.resources.copy(&path)?;
        }
        Ok(())
    }

    fn files(&self, config: &Config) -> Result<Files> {
        let mut files = self.stock.clone();
        let receipt = config.paths.service_path("manifest.json");
        files.remove(&receipt);
        files.insert(
            config::FILE.into(),
            review_runner::config::yaml::encode(config)?.into_bytes(),
        );
        for (path, file) in &self.resources.files {
            files.insert(path.clone(), file.bytes.clone());
        }
        let metadata = serde_json::json!({
            "schema_version": 1, "root": self.resolved.root, "root_digest": self.resolved.root_digest,
            "packages": self.resolved.packages, "provenance": self.resolved.provenance,
            "resources": self.resources.inputs,
            "configurations": self.configurations,
        });
        files.insert(
            config.paths.service_path("composition.json"),
            serde_json::to_vec_pretty(&metadata)?,
        );
        let mut metadata: crate::scaffold::receipt::Manifest =
            serde_json::from_slice(&manifest::installed(&files, config)?)?;
        for (path, file) in &self.resources.files {
            metadata
                .files
                .get_mut(path)
                .context("imported file receipt required")?
                .executable = file.executable;
            let stock = self.stock.contains_key(path);
            if stock {
                metadata.local.insert(
                    path.clone(),
                    Some(crate::scaffold::receipt::checksum(&file.bytes)),
                );
            }
        }
        files.insert(receipt, serde_json::to_vec_pretty(&metadata)?);
        Ok(files)
    }
}

fn unchanged(root: &Path, desired: &Config, files: &Files) -> Result<()> {
    let installed = root.join(config::FILE).is_file();
    if installed {
        ensure!(
            serde_json::to_value(config::read(root)?)? == serde_json::to_value(desired)?,
            "external setup conflicts with installed configuration; use upgrade plan --config CONFIG_YAML to review the update; original preserved"
        );
    }
    let receipt = desired.paths.service_path("composition.json");
    let present = root.join(&receipt).is_file();
    if present {
        ensure!(
            fs::read(root.join(&receipt))? == files[&receipt],
            "composition inputs changed; use upgrade plan --config CONFIG_YAML to review the update; installed environment preserved"
        );
    }
    Ok(())
}
