use serde::{Deserialize, Deserializer, Serialize};

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
    pub fn sandboxed(self) -> bool {
        match self {
            Self::ReadOnly | Self::Project => true,
            Self::Unconfined => false,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Written {
    Named(Access),
    ReadOnly(bool),
}
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Access, D::Error> {
    let access = match Written::deserialize(deserializer)? {
        Written::Named(access) => access,
        Written::ReadOnly(true) => Access::ReadOnly,
        Written::ReadOnly(false) => Access::Unconfined,
    };
    Ok(access)
}
