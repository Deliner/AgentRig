use crate::project_config::Config;
use crate::shared::calling_agent::Agent;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub type Files = BTreeMap<String, Vec<u8>>;
pub type Links = BTreeMap<String, PathBuf>;

// DECISION: D188
pub fn skill_links(config: &Config) -> Links {
    Agent::ALL
        .into_iter()
        .map(|agent| {
            let link = Path::new(agent.skills());
            let depth = link
                .parent()
                .map_or(0, |parent| parent.components().count());
            let target: PathBuf = std::iter::repeat_n("..", depth).collect();
            (agent.skills().to_owned(), target.join(&config.paths.skills))
        })
        .collect()
}
