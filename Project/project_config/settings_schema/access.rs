use serde::{Deserialize, Serialize};

// DECISION: D062
// DECISION: D154
// DECISION: D203
#[derive(Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Access {
    ReadOnly,
    Project,
    #[default]
    Unconfined,
}
impl Access {
    pub fn read_only(self) -> bool {
        match self {
            Self::ReadOnly => true,
            Self::Project | Self::Unconfined => false,
        }
    }
    // DECISION: D213
    pub fn project(self) -> bool {
        match self {
            Self::Project => true,
            Self::ReadOnly | Self::Unconfined => false,
        }
    }
    pub fn sandboxed(self) -> bool {
        match self {
            Self::ReadOnly | Self::Project => true,
            Self::Unconfined => false,
        }
    }
}
