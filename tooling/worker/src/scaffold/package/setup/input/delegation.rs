use super::{Config, Source};
use anyhow::Result;

pub(super) fn prepare(source: &mut Source, config: &mut Config) -> Result<()> {
    let Some(capability) = &mut config.capabilities.delegation else {
        return Ok(());
    };
    let Some(path) = source.path("/capabilities/delegation/config", &capability.config)? else {
        return Ok(());
    };
    let bytes = source.resources.read(&path)?;
    let original: agentrig::delegate::config::Config =
        review_runner::config::yaml::decode(std::str::from_utf8(&bytes)?)?;
    let mut delegation = agentrig::delegate::config::resolve(&path, original)?;
    for profile in delegation.profiles.values_mut() {
        let target = source.resources.copy(&profile.prompt)?;
        profile.prompt = source.resources.sibling(&target)?;
        for skill in &mut profile.skills {
            let target = source.resources.directory(skill)?;
            *skill = source.resources.sibling(&target)?;
        }
        for program in profile.programs.values_mut() {
            let target = source.resources.copy(program)?;
            *program = source.resources.sibling(&target)?;
        }
    }
    capability.config = source.resources.put(
        "profiles.yaml",
        review_runner::config::yaml::encode(&delegation)?.into_bytes(),
        false,
    )?;
    Ok(())
}
