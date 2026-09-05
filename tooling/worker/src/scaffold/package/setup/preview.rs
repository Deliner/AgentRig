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
    let source = fs::read_to_string(preview.join(path))?;
    let policy: crate::lint::config::Config = toml::from_str(&source)?;
    let mut document: toml_edit::DocumentMut = source.parse()?;
    if let Some(skill_root) = policy.skill_root {
        let absolute = root.join(path).parent().unwrap().join(skill_root);
        document["skill_root"] = toml_edit::value(absolute.to_string_lossy().as_ref());
    } else {
        copy(root, preview, &policy.config_skill)?;
        for rule in policy.rules {
            copy(root, preview, &rule.warning_skill)?;
            copy(root, preview, &rule.error_skill)?;
        }
    }
    put(preview, path, document.to_string().as_bytes())
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
        toml::from_str(&fs::read_to_string(preview.join(path))?)?;
    for critic in review.reviewers.values_mut() {
        critic.prompt = resource(root, preview, &parent.join(&critic.prompt))?;
    }
    for (index, tool) in review.tools.values_mut().enumerate() {
        let project_path = resource(root, preview, &parent.join(&tool.project_config))?;
        let mut project: review_runner::config::Project =
            toml::from_str(&fs::read_to_string(&project_path)?)?;
        project.review.contract = resource(
            root,
            preview,
            &project_path
                .parent()
                .unwrap()
                .join(&project.review.contract),
        )?;
        tool.project_config = preview.join(format!(".worker/setup-{index}.toml"));
        fs::write(&tool.project_config, toml::to_string(&project)?)?;
    }
    put(preview, path, toml::to_string(&review)?.as_bytes())
}
