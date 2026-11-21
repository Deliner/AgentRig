use crate::shared::fs_with_context::{self as fs, present};
use anyhow::{Context, Result};
use std::path::Path;

pub fn load_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    let Some(text) = present(fs::read_to_string(path))? else {
        return Ok(None);
    };
    let value = serde_json::from_str(&text)
        .with_context(|| format!("invalid JSON in {}", path.display()))?;
    Ok(Some(value))
}

pub fn save_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    super::replacement::replace(path, &serde_json::to_vec(value)?)
}
