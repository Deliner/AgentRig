use super::{Config, Files, config};
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn validate(root: &Path, config: &Config, files: &Files) -> Result<()> {
    let preview = tempfile::tempdir()?;
    for (name, bytes) in files {
        put(preview.path(), name, bytes)?;
    }
    let mut skills = vec![&config.config_skill];
    skills.extend(config.checks.iter().map(|check| &check.skill));
    skills.extend(config.hooks.routes.iter().map(|route| &route.skill));
    skills.extend(config.hooks.discipline_skill.iter());
    for path in skills {
        copy(root, preview.path(), path)?;
    }
    if let Some(path) = &config.hooks.reminder {
        copy(root, preview.path(), path)?;
    }
    if config.capabilities.lint {
        lint(root, preview.path(), &config.paths.lint)?;
    }
    if let Some(review) = &config.capabilities.review {
        review_config(root, preview.path(), &review.config)?;
    }
    if let Some(delegation) = &config.capabilities.delegation {
        let resolved = agentrig::delegate::config::load(&root.join(&delegation.config))?;
        put(
            preview.path(),
            &delegation.config,
            review_runner::config::yaml::encode(&resolved)?.as_bytes(),
        )?;
    }
    config::Context::load(preview.path())?;
    Ok(())
}
fn put(root: &Path, path: &str, bytes: &[u8]) -> Result<()> {
    let target = config::relative(root, path)?;
    fs::create_dir_all(target.parent().unwrap())?;
    fs::write(target, bytes)?;
    Ok(())
}
fn copy(root: &Path, preview: &Path, path: &str) -> Result<()> {
    let source = config::relative(root, path)?;
    let exists = source.is_file();
    if exists {
        put(preview, path, &fs::read(source)?)?;
    }
    Ok(())
}
fn lint(root: &Path, preview: &Path, path: &str) -> Result<()> {
    copy(root, preview, path)?;
    let mut policy: crate::lint::config::Config =
        review_runner::config::yaml::read(&preview.join(path))?;
    if let Some(skill_root) = &policy.skill_root {
        let absolute = root.join(path).parent().unwrap().join(skill_root);
        policy.skill_root = Some(absolute);
    } else {
        copy(root, preview, &policy.config_skill)?;
        for rule in &policy.rules {
            copy(root, preview, &rule.warning_skill)?;
            copy(root, preview, &rule.error_skill)?;
        }
    }
    put(
        preview,
        path,
        review_runner::config::yaml::encode(&policy)?.as_bytes(),
    )
}
fn resource(root: &Path, preview: &Path, path: &Path) -> Result<PathBuf> {
    let resolved = crate::util::resolve(path)?;
    let exists = resolved.is_file();
    if exists {
        return Ok(resolved);
    }
    Ok(preview.join(resolved.strip_prefix(root)?))
}
fn review_config(root: &Path, preview: &Path, path: &str) -> Result<()> {
    copy(root, preview, path)?;
    let parent = root.join(path).parent().unwrap().to_owned();
    let mut review: review_runner::config::Config =
        review_runner::config::yaml::read(&preview.join(path))?;
    for critic in review.reviewers.values_mut() {
        critic.prompt = resource(root, preview, &parent.join(&critic.prompt))?;
    }
    for (index, tool) in review.tools.values_mut().enumerate() {
        let project_path = resource(root, preview, &parent.join(&tool.project_config))?;
        let mut project: review_runner::config::Project =
            review_runner::config::yaml::read(&project_path)?;
        project.review.contract = resource(
            root,
            preview,
            &project_path
                .parent()
                .unwrap()
                .join(&project.review.contract),
        )?;
        tool.project_config = preview.join(format!(".worker/setup-{index}.yaml"));
        fs::write(
            &tool.project_config,
            review_runner::config::yaml::encode(&project)?,
        )?;
    }
    put(
        preview,
        path,
        review_runner::config::yaml::encode(&review)?.as_bytes(),
    )
}
