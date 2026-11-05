use crate::shared::fs;
use anyhow::{Result, bail};
use std::path::Path;

pub fn reference(root: &Path, value: &str) -> Result<()> {
    let absolute = Path::new(value).is_absolute();
    if absolute {
        bail!("skill paths must be relative to the repository: {value}");
    }
    let path = fs::canonicalize(root.join(value))?;
    let invalid_location =
        !path.starts_with(root) || path.file_name().is_none_or(|name| name != "SKILL.md");
    if invalid_location {
        bail!("skill must reference a repository SKILL.md: {value}");
    }
    let source = fs::read_to_string(path)?;
    let invalid_frontmatter = !source.starts_with("---\n")
        || !source.contains("\nname: ")
        || !source.contains("\ndescription: ");
    if invalid_frontmatter {
        bail!("invalid skill frontmatter: {value}");
    }
    Ok(())
}
