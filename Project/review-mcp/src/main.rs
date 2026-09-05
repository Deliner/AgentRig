use anyhow::{Result, bail};
use std::{env, path::Path};
fn run() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [command, path] if command == "mcp" => review_runner::mcp::serve(Path::new(path)),
        [command] if command == "hook" => review_runner::execution::broker::hook(),
        [command, config, request] if command == "run" => {
            let request = serde_json::from_slice(&std::fs::read(request)?)?;
            let report = review_runner::run::run(Path::new(config), request)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        [command, path] if command == "config-check" => {
            review_runner::config::load(Path::new(path))?;
            println!("Review configuration and contracts are valid");
            Ok(())
        }
        _ => {
            bail!("review-runner config-check CONFIG | run CONFIG REQUEST_JSON | mcp CONFIG | hook")
        }
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(2);
    }
}
