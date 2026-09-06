use super::{Config, Source};
use anyhow::Result;

pub(super) fn prepare(source: &mut Source, config: &mut Config) -> Result<()> {
    let Some(capability) = &mut config.capabilities.review else {
        return Ok(());
    };
    let Some(path) = source.path("/capabilities/review/config", &capability.config)? else {
        return Ok(());
    };
    let bytes = source.resources.read(&path)?;
    let original: review_runner::config::Config =
        review_runner::config::yaml::decode(std::str::from_utf8(&bytes)?)?;
    let runtime = original.runner.runtime_root.clone();
    let reports = original.runner.report_root.clone();
    let mut review = review_runner::config::resolve(&path, original)?;
    review.runner.runtime_root = runtime;
    review.runner.report_root = reports;
    for reviewer in review.reviewers.values_mut() {
        let target = source.resources.copy(&reviewer.prompt)?;
        reviewer.prompt = source.resources.sibling(&target)?;
    }
    for tool in review.tools.values_mut() {
        let target = project(source, &tool.project_config)?;
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
    let bytes = source.resources.read(path)?;
    let mut project: review_runner::config::Project =
        review_runner::config::yaml::decode(std::str::from_utf8(&bytes)?)?;
    let contract = review_runner::config::resource(path, &project.review.contract)?;
    let target = source.resources.copy(&contract)?;
    project.review.contract = source.resources.sibling(&target)?;
    source.resources.put(
        "project.yaml",
        review_runner::config::yaml::encode(&project)?.into_bytes(),
        false,
    )
}
