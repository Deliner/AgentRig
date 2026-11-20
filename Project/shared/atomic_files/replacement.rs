use anyhow::Result;
use std::io::Write as _;
use std::path::Path;

pub fn replace(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("a replaced file needs a parent"))?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.as_file_mut().write_all(content)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path)?;
    Ok(())
}
