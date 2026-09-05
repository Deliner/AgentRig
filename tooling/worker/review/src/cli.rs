use anyhow::{Result, bail};
use std::path::Path;
pub fn run(args: &[String]) -> Result<()> {
    match args {
        [command, path] if command == "mcp" => crate::mcp::serve(Path::new(path)),
        [command] if matches!(command.as_str(), "hook" | "review-hook") => {
            crate::execution::broker::hook()
        }
        [command, config, request] if command == "run" => {
            let request = serde_json::from_slice(&std::fs::read(request)?)?;
            let report = crate::run::run(Path::new(config), request)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        [command, path] if command == "config-check" => {
            crate::config::load(Path::new(path))?;
            println!("Review configuration and contracts are valid");
            Ok(())
        }
        _ => {
            bail!("review-runner config-check CONFIG | run CONFIG REQUEST_JSON | mcp CONFIG | hook")
        }
    }
}
