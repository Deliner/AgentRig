use crate::project_config::Context;
use anyhow::Result;
use std::path::PathBuf;

pub(super) fn mirror(context: &Context) -> Result<PathBuf> {
    Ok(context
        .path(&context.config.paths.runtime)?
        .join("proposal"))
}
