use super::pid_reuse_guard::Identity;
use crate::project_config::{Access, Lifetime};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::PathBuf;

// DECISION: D062
// DECISION: D154
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Grouping {
    Own,
    Inherited,
}
impl Grouping {
    pub fn target(self, pid: i32) -> i32 {
        match self {
            Self::Own => -pid,
            Self::Inherited => pid,
        }
    }
}
mod process_group {
    use super::{Deserialize, Deserializer, Grouping, Serializer};
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Grouping, D::Error> {
        let own = bool::deserialize(deserializer)?;
        Ok(if own {
            Grouping::Own
        } else {
            Grouping::Inherited
        })
    }
    pub fn serialize<S: Serializer>(grouping: &Grouping, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(match grouping {
            Grouping::Own => true,
            Grouping::Inherited => false,
        })
    }
}

#[derive(Deserialize, Serialize)]
pub struct Record {
    pub run_id: String,
    pub owner: String,
    pub parent_run: Option<String>,
    pub project: PathBuf,
    pub cwd: PathBuf,
    pub branch: Option<String>,
    pub command: String,
    pub argv: Vec<String>,
    pub started: u64,
    pub finished: Option<u64>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub supervisor: Identity,
    pub child: Option<Identity>,
    #[serde(
        rename = "process_group",
        deserialize_with = "process_group::deserialize",
        serialize_with = "process_group::serialize"
    )]
    pub grouping: Grouping,
    #[serde(default)]
    pub duration_seconds: Option<f64>,
    #[serde(default)]
    pub group: Option<Identity>,
    #[serde(default)]
    pub access: Access,
    #[serde(default)]
    pub lifetime: Lifetime,
    #[serde(default)]
    pub background: bool,
}
