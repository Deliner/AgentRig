use super::config::Config;
use anyhow::{Result, ensure};
use std::path::{Component, Path, PathBuf};

// DECISION: D038
pub const FILE: &str = ".agentrig/project.yaml";
const MODULES: [&str; 7] = [
    "git", "commands", "checks", "hooks", "oracles", "tools", "rust",
];

pub fn module(section: &str) -> String {
    format!(".agentrig/{section}.yaml")
}
pub fn read(root: &Path) -> Result<Config> {
    let mut sections: serde_json::Map<String, serde_json::Value> =
        crate::shared::strict_yaml::read(&root.join(FILE))?;
    for section in MODULES {
        let path = module(section);
        ensure!(
            !sections.contains_key(section),
            "{FILE}: move {section} to {path}"
        );
        let present = root.join(&path).is_file();
        if present {
            sections.insert(
                section.into(),
                crate::shared::strict_yaml::read(&root.join(&path))?,
            );
        }
    }
    serde_path_to_error::deserialize(serde_json::Value::Object(sections)).map_err(|error| {
        let field = error.path().to_string();
        let section = field.split(['.', '[']).next().unwrap_or_default();
        let owned = MODULES.contains(&section);
        let file = if owned { module(section) } else { FILE.into() };
        anyhow::anyhow!("{file}: {error}")
    })
}
// DECISION: D045
pub fn sources(root: &Path) -> Vec<PathBuf> {
    let mut sources: Vec<PathBuf> = std::iter::once(FILE.to_owned())
        .chain(MODULES.map(module))
        .map(PathBuf::from)
        .collect();
    for (file, pointer) in [
        (FILE.to_owned(), "/paths/lint"),
        (module("hooks"), "/reminder"),
    ] {
        let document = crate::shared::strict_yaml::read::<serde_json::Value>(&root.join(file)).ok();
        let referenced = document
            .as_ref()
            .and_then(|document| document.pointer(pointer))
            .and_then(serde_json::Value::as_str)
            .and_then(|path| relative(root, path).ok())
            .and_then(|path| path.strip_prefix(root).ok().map(Path::to_owned));
        sources.extend(referenced);
    }
    sources
}
pub struct ModuleFile {
    pub path: String,
    pub source: String,
}
pub fn files(config: &Config) -> Result<Vec<ModuleFile>> {
    let serde_json::Value::Object(mut sections) = serde_json::to_value(config)? else {
        anyhow::bail!("configuration must serialize to a mapping")
    };
    let mut files = Vec::new();
    for section in MODULES {
        let value = sections
            .remove(section)
            .ok_or_else(|| anyhow::anyhow!("missing configuration section {section}"))?;
        files.push(ModuleFile {
            path: module(section),
            source: crate::shared::strict_yaml::encode(&value)?,
        });
    }
    files.push(ModuleFile {
        path: FILE.into(),
        source: crate::shared::strict_yaml::encode(&sections)?,
    });
    Ok(files)
}
pub fn project_root(path: &Path) -> Result<PathBuf> {
    Ok(crate::shared::fs_with_context::canonicalize(path)?)
}
pub fn relative(root: &Path, value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    ensure!(
        !value.is_empty()
            && !path.is_absolute()
            && !path.components().any(|c| matches!(c, Component::ParentDir)),
        "path must stay relative to project: {value}"
    );
    let resolved = crate::shared::paths::resolve(&root.join(path))?;
    ensure!(resolved.starts_with(root), "path escapes project: {value}");
    Ok(resolved)
}
