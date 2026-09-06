use super::{Config, Source};
use anyhow::{Context, Result};

pub(super) fn prepare(source: &mut Source, config: &mut Config) -> Result<()> {
    let disabled = !config.capabilities.lint;
    if disabled {
        return Ok(());
    }
    let Some(path) = source.path("/paths/lint", &config.paths.lint)? else {
        return Ok(());
    };
    let bytes = source.resources.read(&path)?;
    let mut policy: crate::lint::config::Config =
        review_runner::config::yaml::decode(std::str::from_utf8(&bytes)?)?;
    let parent = source
        .resolved
        .origin("/paths/lint")
        .parent()
        .context("lint source directory required")?;
    let skill_root = match &policy.skill_root {
        Some(root) => path
            .parent()
            .context("lint configuration directory required")?
            .join(root),
        None => parent.to_owned(),
    };
    policy.config_skill = skill(source, &skill_root, &policy.config_skill)?;
    for rule in &mut policy.rules {
        rule.warning_skill = skill(source, &skill_root, &rule.warning_skill)?;
        rule.error_skill = skill(source, &skill_root, &rule.error_skill)?;
    }
    policy.skill_root = None;
    source.stock.remove(&config.paths.lint);
    config.paths.lint = source.resources.put(
        "lint.yaml",
        review_runner::config::yaml::encode(&policy)?.into_bytes(),
        false,
    )?;
    Ok(())
}

fn skill(source: &mut Source, root: &std::path::Path, value: &str) -> Result<String> {
    let path = root.join(value);
    let shipped = !path.exists() && source.stock.contains_key(value);
    if shipped {
        return Ok(value.into());
    }
    source.materialize_skill(&path, value)
}
