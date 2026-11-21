mod repair_guidance;
mod schema;
mod validation;

pub use schema::{Config, Rule};
use std::path::Path;
pub use validation::Thresholds;

pub fn extension_matches(path: &Path, extensions: &[String]) -> bool {
    extensions.is_empty()
        || path.extension().is_some_and(|ext| {
            extensions
                .iter()
                .any(|wanted| wanted == &format!(".{}", ext.to_string_lossy()))
        })
}
