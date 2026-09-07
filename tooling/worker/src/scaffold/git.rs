// DECISION: D010
use super::{config::Context, gate};
use anyhow::{Result, ensure};
use std::process::Command;

pub fn start(context: &Context, name: &str) -> Result<i32> {
    let settings = &context.config.vcs;
    let repository = settings
        .backend
        .repository(&context.root)?
        .ok_or_else(|| anyhow::anyhow!("feature-start requires a VCS repository"))?;
    let branch = repository.start_feature(settings, name)?;
    println!("{branch}");
    Ok(0)
}

pub fn merge(context: &Context) -> Result<i32> {
    let settings = &context.config.vcs;
    let repository = settings
        .backend
        .repository(&context.root)?
        .ok_or_else(|| anyhow::anyhow!("integration requires a VCS repository"))?;
    let (feature, code) = repository.integrate(settings, || checked(context), execute_vcs)?;
    let passed = code == 0;
    if passed {
        println!("merged {feature}; branch retained");
        return cleanup_merged(context, &feature);
    }
    Ok(code)
}

fn checked(context: &Context) -> Result<i32> {
    let code = gate::run(&context.root, false)?;
    let passed = code == 0;
    if passed {
        ensure!(
            super::evidence::resume(context)["full_gate_passed"] == true,
            "checked merge inputs changed; preserve the native operation and retry after repairing the checks"
        );
    }
    Ok(code)
}

fn execute_vcs(command: &mut Command) -> Result<i32> {
    let output = agentrig::jobs::process::execute(command, false, None)?;
    Ok(super::process::exit_code(&output))
}

fn cleanup_merged(context: &Context, feature: &str) -> Result<i32> {
    let identified = agentrig::jobs::owner().is_some();
    let anonymous = !identified;
    if anonymous {
        return Ok(0);
    }
    let result =
        agentrig::jobs::cleanup::run(&context.path(&context.config.paths.runtime)?, Some(feature));
    match result {
        Ok(report) => {
            let failed = report["errors"]
                .as_array()
                .is_some_and(|errors| !errors.is_empty());
            println!("post-merge cleanup: {report}");
            if failed {
                eprintln!("merge succeeded; cleanup failed. Retry job-cleanup --branch {feature}");
            }
            Ok(i32::from(failed))
        }
        Err(error) => {
            eprintln!(
                "merge succeeded; cleanup failed: {error}. Retry job-cleanup --branch {feature}"
            );
            Ok(1)
        }
    }
}
