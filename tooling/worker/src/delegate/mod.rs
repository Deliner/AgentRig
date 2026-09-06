pub mod config;
pub mod task;

use anyhow::{Result, bail};
use std::path::Path;

pub fn cli(root: &Path, args: &[String]) -> Result<i32> {
    match args {
        [command, path] if command == "config-check" => {
            let config = config::load(&root.join(path))?;
            println!(
                "Delegation configuration is valid ({} profiles)",
                config.profiles.len()
            );
            Ok(0)
        }
        _ => bail!("delegate config-check CONFIG"),
    }
}
