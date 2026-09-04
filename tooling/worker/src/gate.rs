use crate::lint::config;
use anyhow::{Result, bail};
use std::{path::Path, process::Command};

// DECISION: D016
pub fn run(root: &Path, args: &[String]) -> Result<i32> {
    if args.len() < 3 || args[1] != "--" {
        bail!("gate STAGE -- COMMAND [ARGS]");
    }
    let config = match config::load(root, &root.join("tooling/worker/lint.toml")) {
        Ok(config) => config,
        Err(error) => {
            eprintln!(
                "ERROR [configuration]: {error:#}. ACTION: Apply .agents/skills/configure-linter/SKILL.md"
            );
            return Ok(2);
        }
    };
    let Some(skill) = config.gate_skills.get(&args[0]) else {
        eprintln!(
            "ERROR [configuration]: no repair skill for gate stage {}. ACTION: Apply {}",
            args[0], config.config_skill
        );
        return Ok(2);
    };
    let code = Command::new(&args[2])
        .args(&args[3..])
        .current_dir(root)
        .status()
        .map(|status| status.code().unwrap_or(1));
    let code = match code {
        Ok(code) => code,
        Err(error) => {
            eprintln!("cannot execute {}: {error}", args[2]);
            127
        }
    };
    if code != 0 {
        eprintln!("ERROR [{}]: check failed. ACTION: Apply {skill}", args[0]);
    }
    Ok(code)
}
