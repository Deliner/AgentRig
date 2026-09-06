use super::{
    Config,
    registration::{setting, table},
};
use anyhow::Result;
use toml_edit::{DocumentMut, value};

pub(super) fn configure(config: &Config, document: &mut DocumentMut) -> Result<()> {
    for (name, server) in &config.environment.mcp_servers {
        table(&mut document["mcp_servers"], "mcp_servers")?;
        let target = &mut document["mcp_servers"][name];
        table(target, &format!("mcp_servers.{name}"))?;
        setting(&mut target["enabled"], value(true), name)?;
        setting(&mut target["command"], value("sh"), name)?;
        let mut args = toml_edit::Array::new();
        args.push("-c");
        args.push(super::super::adapters::environment_command(
            config, "mcp", name,
        ));
        setting(&mut target["args"], value(args), name)?;
        let mut references = toml_edit::Array::new();
        let names: std::collections::BTreeSet<_> = server.env.values().collect();
        for name in names {
            references.push(name.as_str());
        }
        setting(&mut target["env_vars"], value(references), name)?;
    }
    Ok(())
}
