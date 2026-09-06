mod files;
mod result;
#[cfg(test)]
mod tests;

use super::config::{Mode, Profile};
use anyhow::{Context, Result, ensure};
pub use result::verify;
use review_runner::{config::Repository, snapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub profile: String,
    pub task: String,
    pub revision: Option<String>,
    #[serde(default)]
    pub inputs: BTreeMap<String, String>,
    pub contract: Contract,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub result_schema: Value,
    #[serde(default)]
    pub artifacts: BTreeMap<String, u64>,
    pub changes: Option<Changes>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Changes {
    pub write_paths: Vec<String>,
    pub checks: BTreeMap<String, Vec<String>>,
}
#[derive(Deserialize, Serialize)]
pub struct Inputs {
    pub revision: Option<String>,
    pub manifest: BTreeMap<String, String>,
}

pub fn validate(request: &Request, profile: &Profile) -> Result<()> {
    ensure!(!request.task.trim().is_empty(), "task must not be empty");
    validator(&request.contract)?;
    code_contract(request, profile)?;
    let reading = matches!(profile.mode, Mode::Read);
    ensure!(
        !reading || request.contract.artifacts.is_empty(),
        "read mode cannot return artifacts"
    );
    for (name, source) in &request.inputs {
        files::relative(name)?;
        files::relative(source)?;
    }
    Ok(())
}

fn code_contract(request: &Request, profile: &Profile) -> Result<()> {
    let coding = matches!(profile.mode, Mode::Code);
    ensure!(
        coding == request.contract.changes.is_some(),
        "code mode requires changes; other modes forbid changes"
    );
    if let Some(changes) = &request.contract.changes {
        ensure!(request.revision.is_some(), "code mode requires revision");
        review_runner::config::globs(&changes.write_paths)?;
        ensure!(!changes.checks.is_empty(), "code mode requires checks");
        for (name, argv) in &changes.checks {
            ensure!(
                review_runner::config::identifier(name),
                "invalid code check name"
            );
            ensure!(
                argv.first()
                    .is_some_and(|program| profile.programs.contains_key(program)),
                "code check {name} must start with a configured program name"
            );
        }
    }
    Ok(())
}

fn validator(contract: &Contract) -> Result<jsonschema::Validator> {
    for (path, limit) in &contract.artifacts {
        files::relative(path)?;
        ensure!(
            path != "result.json",
            "result.json is reserved for the response"
        );
        ensure!(*limit > 0, "artifact {path} requires a positive byte limit");
    }
    jsonschema::validator_for(&contract.result_schema).context("invalid result schema")
}

pub fn prepare(
    root: &Path,
    directory: &Path,
    request: &Request,
    profile: &Profile,
) -> Result<Inputs> {
    validate(request, profile)?;
    fs::create_dir(directory).context("task input directory must be new")?;
    let mut inputs = prepare_project(root, &directory.join("project"), request, profile)?;
    let explicit = directory.join("inputs");
    fs::create_dir(&explicit)?;
    files::copy_inputs((root, &explicit), request, profile, &mut inputs.manifest)?;
    fs::write(
        directory.join("request.json"),
        serde_json::to_vec_pretty(request)?,
    )?;
    fs::write(
        directory.join("schema.json"),
        serde_json::to_vec_pretty(&request.contract.result_schema)?,
    )?;
    fs::write(
        directory.join("manifest.json"),
        serde_json::to_vec_pretty(&inputs)?,
    )?;
    Ok(inputs)
}

fn prepare_project(
    root: &Path,
    project: &Path,
    request: &Request,
    profile: &Profile,
) -> Result<Inputs> {
    let mut inputs = Inputs {
        revision: None,
        manifest: BTreeMap::new(),
    };
    if let Some(revision) = &request.revision {
        let scope = Repository {
            visible_paths: profile.visible_paths.clone(),
            contract_paths: vec![],
        };
        let snapshot = snapshot::prepare(root, (revision, revision), &scope, project)?;
        inputs.revision = Some(snapshot.candidate);
        for (path, entry) in snapshot.manifest {
            inputs
                .manifest
                .insert(format!("project/{path}"), entry.sha256);
        }
    } else {
        fs::create_dir(project)?;
    }
    Ok(inputs)
}

pub fn path(root: &Path, relative: &str) -> Result<PathBuf> {
    files::relative(relative)?;
    let mut path = root.to_path_buf();
    for component in Path::new(relative).components() {
        path.push(component);
        let metadata = fs::symlink_metadata(&path)?;
        ensure!(
            !metadata.file_type().is_symlink(),
            "symlink is not an allowed task file: {relative}"
        );
    }
    Ok(path)
}
