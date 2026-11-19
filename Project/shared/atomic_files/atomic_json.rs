use crate::shared::fs_with_context::{self as fs, present};
use anyhow::{Context, Result};
use std::path::Path;

pub fn load<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    let Some(text) = present(fs::read_to_string(path))? else {
        return Ok(None);
    };
    let value = serde_json::from_str(&text)
        .with_context(|| format!("invalid JSON in {}", path.display()))?;
    Ok(Some(value))
}

pub fn save(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("JSON path needs a parent"))?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer(temporary.as_file_mut(), value)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path)?;
    Ok(())
}
