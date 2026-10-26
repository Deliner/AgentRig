//! Atomically replace a JSON artifact after serialization and file synchronization.
use anyhow::Result;
use std::path::Path;

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
