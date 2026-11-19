use serde::{Deserialize, Deserializer, Serializer, de::Error};

// DECISION: D202
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Arguments {
    #[default]
    None,
    Operands,
    Any,
}
impl Arguments {
    pub fn accepted(self) -> bool {
        match self {
            Self::None => false,
            Self::Operands | Self::Any => true,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Written {
    Accepted(bool),
    Kind(String),
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Arguments, D::Error> {
    match Written::deserialize(deserializer)? {
        Written::Accepted(true) => Ok(Arguments::Any),
        Written::Accepted(false) => Ok(Arguments::None),
        Written::Kind(kind) if kind == "operands" => Ok(Arguments::Operands),
        Written::Kind(kind) => Err(D::Error::custom(format!(
            "accepts_args is true, false or operands, not {kind:?}"
        ))),
    }
}

pub fn serialize<S: Serializer>(arguments: &Arguments, serializer: S) -> Result<S::Ok, S::Error> {
    match arguments {
        Arguments::None => serializer.serialize_bool(false),
        Arguments::Any => serializer.serialize_bool(true),
        Arguments::Operands => serializer.serialize_str("operands"),
    }
}
