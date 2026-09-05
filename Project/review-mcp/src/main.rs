use anyhow::{Result, bail};
use std::{env, path::Path};
fn run() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [command, path] if command == "config-check" => {
            review_runner::config::load(Path::new(path))?;
            println!("Review configuration and contracts are valid");
            Ok(())
        }
        _ => bail!("review-runner config-check CONFIG"),
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(2);
    }
}
