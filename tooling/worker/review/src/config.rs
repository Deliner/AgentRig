pub mod yaml;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub runner: Runner,
    pub reviewers: BTreeMap<String, Reviewer>,
    pub tools: BTreeMap<String, Tool>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Runner {
    pub runtime_root: PathBuf,
    pub report_root: PathBuf,
    pub isolation: String,
    pub parallelism: usize,
    pub timeout_seconds: u64,
    pub format_attempts: usize,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Reviewer {
    #[serde(default = "codex")]
    pub frontend: String,
    pub model: String,
    pub reasoning_effort: String,
    pub prompt: PathBuf,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    pub description: String,
    pub reviewers: Vec<String>,
    pub project_config: PathBuf,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub schema_version: u32,
    pub repository: Repository,
    pub review: Review,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    #[serde(default)]
    pub vcs: crate::vcs::Kind,
    pub visible_paths: Vec<String>,
    pub contract_paths: Vec<String>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub contract: PathBuf,
}

pub fn resource(config: &Path, resource: &Path) -> Result<PathBuf> {
    config
        .parent()
        .context("configuration needs a parent")?
        .join(resource)
        .canonicalize()
        .with_context(|| {
            format!(
                "resolve resource {} relative to {}",
                resource.display(),
                config.display()
            )
        })
}
pub fn load(path: &Path) -> Result<Config> {
    resolve(path, yaml::read(path)?)
}
pub fn resolve(path: &Path, mut config: Config) -> Result<Config> {
    ensure!(
        config.schema_version == 1,
        "unsupported configuration schema"
    );
    validate_runner(&config.runner)?;
    ensure!(!config.tools.is_empty(), "at least one tool is required");
    for (name, reviewer) in &mut config.reviewers {
        validate_reviewer(name, reviewer)?;
        reviewer.prompt = resource(path, &reviewer.prompt)?;
        fs::read_to_string(&reviewer.prompt)?;
    }
    for (name, tool) in &mut config.tools {
        validate_tool(name, tool, &config.reviewers)?;
        tool.project_config = resource(path, &tool.project_config)?;
        let project = project(&tool.project_config)?;
        crate::contract::load(&project.review.contract, &tool.reviewers)?;
    }
    let parent = path.parent().context("configuration needs a parent")?;
    config.runner.runtime_root = parent.join(&config.runner.runtime_root);
    config.runner.report_root = parent.join(&config.runner.report_root);
    ensure!(
        config.runner.runtime_root != config.runner.report_root,
        "runtime and report roots must differ"
    );
    Ok(config)
}
fn codex() -> String {
    "codex".into()
}
fn validate_reviewer(name: &str, reviewer: &Reviewer) -> Result<()> {
    ensure!(
        reviewer.frontend == "codex",
        "unsupported frontend {:?} for reviewer {name}; supported: codex",
        reviewer.frontend
    );
    ensure!(
        identifier(name) && !reviewer.model.trim().is_empty(),
        "invalid reviewer {name}"
    );
    reasoning_effort(&reviewer.reasoning_effort)?;
    Ok(())
}
pub fn reasoning_effort(value: &str) -> Result<()> {
    ensure!(
        ["minimal", "low", "medium", "high", "xhigh"].contains(&value),
        "unsupported reasoning effort {value}"
    );
    Ok(())
}
fn validate_runner(runner: &Runner) -> Result<()> {
    ensure!(
        runner.isolation == "bubblewrap",
        "only bubblewrap isolation is implemented"
    );
    ensure!(
        runner.parallelism > 0 && runner.timeout_seconds > 0 && runner.format_attempts > 0,
        "parallelism, timeout_seconds and format_attempts must be positive"
    );
    Ok(())
}
fn validate_tool(name: &str, tool: &Tool, reviewers: &BTreeMap<String, Reviewer>) -> Result<()> {
    ensure!(
        identifier(name) && !tool.description.trim().is_empty(),
        "invalid tool {name}"
    );
    ensure!(!tool.reviewers.is_empty(), "tool {name} needs reviewers");
    let unique: BTreeSet<_> = tool.reviewers.iter().collect();
    ensure!(
        unique.len() == tool.reviewers.len(),
        "duplicate reviewers in {name}"
    );
    for role in &tool.reviewers {
        ensure!(
            reviewers.contains_key(role),
            "unknown reviewer {role} in {name}"
        );
    }
    Ok(())
}
pub fn project(path: &Path) -> Result<Project> {
    let mut project: Project = yaml::read(path)?;
    ensure!(project.schema_version == 1, "unsupported project schema");
    ensure!(
        !project.repository.visible_paths.is_empty(),
        "visible_paths cannot be empty"
    );
    globs(&project.repository.visible_paths)?;
    globs(&project.repository.contract_paths)?;
    project.review.contract = resource(path, &project.review.contract)?;
    Ok(project)
}
pub fn globs(patterns: &[String]) -> Result<globset::GlobSet> {
    let mut builder = globset::GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(globset::Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}
pub fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
}
