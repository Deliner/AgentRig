use super::{Config, Source};
use anyhow::Result;

pub(super) fn prepare(source: &mut Source, config: &mut Config) -> Result<()> {
    let Some(capability) = &mut config.capabilities.review else {
        return Ok(());
    };
    let Some(path) = source.path("/capabilities/review/config", &capability.config)? else {
        return Ok(());
    };
    let mut review: review_runner::config::Config = source.configuration(&path)?;
    for (name, reviewer) in &mut review.reviewers {
        let prompt = source.resource(
            &path,
            &format!("/reviewers/{name}/prompt"),
            &reviewer.prompt,
        )?;
        let target = source.resources.copy(&prompt)?;
        reviewer.prompt = source.resources.sibling(&target)?;
    }
    for (name, tool) in &mut review.tools {
        let resource = source.resource(
            &path,
            &format!("/tools/{name}/project_config"),
            &tool.project_config,
        )?;
        let target = project(source, &resource)?;
        tool.project_config = source.resources.sibling(&target)?;
    }
    capability.config = source.resources.put(
        "review.yaml",
        review_runner::config::yaml::encode(&review)?.into_bytes(),
        false,
    )?;
    Ok(())
}

fn project(source: &mut Source, path: &std::path::Path) -> Result<String> {
    let mut project: review_runner::config::Project = source.configuration(path)?;
    let contract = source.resource(path, "/review/contract", &project.review.contract)?;
    let target = source.resources.copy(&contract)?;
    project.review.contract = source.resources.sibling(&target)?;
    source.resources.put(
        "project.yaml",
        review_runner::config::yaml::encode(&project)?.into_bytes(),
        false,
    )
}
