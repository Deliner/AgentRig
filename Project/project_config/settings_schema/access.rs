use serde::{Deserialize, Deserializer, Serializer};

// DECISION: D062
// DECISION: D154
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Access {
    ReadOnly,
    #[default]
    Writable,
}
impl Access {
    pub fn read_only(self) -> bool {
        match self {
            Self::ReadOnly => true,
            Self::Writable => false,
        }
    }
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Access, D::Error> {
    let read_only = bool::deserialize(deserializer)?;
    Ok(if read_only {
        Access::ReadOnly
    } else {
        Access::Writable
    })
}

pub fn serialize<S: Serializer>(access: &Access, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bool(access.read_only())
}
