use super::{Config, Source};
use anyhow::Result;

pub(super) fn prepare(source: &mut Source, config: &mut Config) -> Result<()> {
    let Some(capability) = &mut config.capabilities.delegation else {
        return Ok(());
    };
    let Some(path) = source.path("/capabilities/delegation/config", &capability.config)? else {
        return Ok(());
    };
    let mut delegation: agentrig::delegate::config::Config = source.configuration(&path)?;
    for (name, profile) in &mut delegation.profiles {
        let address = format!("/profiles/{name}");
        let prompt = source.resource(&path, &format!("{address}/prompt"), &profile.prompt)?;
        let target = source.resources.copy(&prompt)?;
        profile.prompt = source.resources.sibling(&target)?;
        for skill in &mut profile.skills {
            let resource = source.resource(&path, &format!("{address}/skills"), skill)?;
            let target = source.resources.directory(&resource)?;
            *skill = source.resources.sibling(&target)?;
        }
        for (name, program) in &mut profile.programs {
            let resource =
                source.resource(&path, &format!("{address}/programs/{name}"), program)?;
            let target = source.resources.copy(&resource)?;
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
