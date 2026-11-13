use super::{Python, module_parts};
use anyhow::{Context, Result, ensure};
use std::path::Path;

impl Python<'_> {
    pub(super) fn absolute(&self, source: &Path, module: &str) -> Result<String> {
        let depth = module.bytes().take_while(|byte| *byte == b'.').count();
        let absolute = depth == 0;
        if absolute {
            return Ok(module.into());
        }
        let parent = source.parent().context("Python source has no parent")?;
        let package = parent.strip_prefix(&self.root)?;
        let mut parts: Vec<_> = package.iter().map(|part| part.to_string_lossy()).collect();
        ensure!(
            depth <= parts.len(),
            "relative import escapes its package: {module}"
        );
        parts.truncate(parts.len() + 1 - depth);
        let suffix = module.trim_start_matches('.');
        let has_suffix = !suffix.is_empty();
        if has_suffix {
            parts.extend(module_parts(suffix)?.into_iter().map(Into::into));
        }
        Ok(parts.join("."))
    }
}
