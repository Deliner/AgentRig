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
    let mut policy: crate::lint::config::Config = source.configuration(&path)?;
    skills(source, &path, &mut policy)?;
    policy.skill_root = None;
    source.stock.remove(&config.paths.lint);
    config.paths.lint = source.resources.put(
        "lint.yaml",
        review_runner::config::yaml::encode(&policy)?.into_bytes(),
        false,
    )?;
    Ok(())
}

fn skills(
    source: &mut Source,
    path: &std::path::Path,
    policy: &mut crate::lint::config::Config,
) -> Result<()> {
    let roots = skill_roots(source, path, policy.skill_root.as_deref())?;
    policy.config_skill = skill(
        source,
        (path, "/config_skill"),
        &roots,
        &policy.config_skill,
    )?;
    for rule in &mut policy.rules {
        let id = rule.id.replace('~', "~0").replace('/', "~1");
        rule.warning_skill = skill(
            source,
            (path, &format!("/rules/{id}/warning_skill")),
            &roots,
            &rule.warning_skill,
        )?;
        rule.error_skill = skill(
            source,
            (path, &format!("/rules/{id}/error_skill")),
            &roots,
            &rule.error_skill,
        )?;
    }
    Ok(())
}

fn skill_roots(
    source: &Source,
    path: &std::path::Path,
    explicit: Option<&std::path::Path>,
) -> Result<(std::path::PathBuf, Option<std::path::PathBuf>)> {
    let parent = source
        .resolved
        .origin("/paths/lint")
        .parent()
        .context("lint source directory required")?;
    let skill_root = explicit.map(|root| {
        source.configurations[path]
            .origin("/skill_root")
            .parent()
            .unwrap()
            .join(root)
    });
    Ok((parent.to_owned(), skill_root))
}

fn skill(
    source: &mut Source,
    reference: (&std::path::Path, &str),
    roots: &(std::path::PathBuf, Option<std::path::PathBuf>),
    value: &str,
) -> Result<String> {
    let configuration = &source.configurations[reference.0];
    let declaring = configuration.origin(reference.1);
    let imported = declaring != configuration.root;
    let relative = if imported {
        declaring.parent().unwrap()
    } else {
        &roots.0
    };
    let path = roots.1.as_deref().unwrap_or(relative).join(value);
    let shipped = !path.exists() && source.stock.contains_key(value);
    if shipped {
        return Ok(value.into());
    }
    source.materialize_skill(&path, value)
}
