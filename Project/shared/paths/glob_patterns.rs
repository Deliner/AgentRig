use anyhow::{Result, bail};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::path::Path;

pub fn globs(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let empty = pattern.trim().is_empty();
        if empty {
            bail!("empty glob");
        }
        let escapes_root =
            Path::new(pattern).is_absolute() || pattern.split('/').any(|part| part == "..");
        if escapes_root {
            bail!("globs must stay relative to the repository: {pattern}");
        }
        builder.add(GlobBuilder::new(pattern).literal_separator(true).build()?);
    }
    Ok(builder.build()?)
}
