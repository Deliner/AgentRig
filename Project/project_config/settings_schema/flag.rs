use anyhow::{Error, Result, ensure};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use std::fmt::{self, Display, Formatter};

// DECISION: D202
pub struct Flag {
    form: Form,
}
enum Form {
    Long { name: String },
    Short { letter: char },
    Word { name: String },
}
impl TryFrom<String> for Flag {
    type Error = Error;
    fn try_from(text: String) -> Result<Self> {
        let long = text.strip_prefix("--");
        let name = long.or(text.strip_prefix('-')).unwrap_or_default();
        let valid = name.starts_with(|ch: char| ch.is_ascii_alphanumeric())
            && name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-');
        ensure!(
            valid,
            "deny entry {text:?} is not a flag; write --name, -c or -name"
        );
        let mut letters = name.chars();
        let form = match (long, letters.next(), letters.next()) {
            (Some(_), ..) => Form::Long { name: name.into() },
            (None, Some(letter), None) => Form::Short { letter },
            (None, ..) => Form::Word { name: name.into() },
        };
        Ok(Self { form })
    }
}
impl Display for Flag {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match &self.form {
            Form::Long { name } => write!(formatter, "--{name}"),
            Form::Short { letter } => write!(formatter, "-{letter}"),
            Form::Word { name } => write!(formatter, "-{name}"),
        }
    }
}
impl<'de> Deserialize<'de> for Flag {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::try_from(text).map_err(D::Error::custom)
    }
}
impl Serialize for Flag {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
impl Flag {
    // DECISION: D202
    pub fn denies(&self, word: &str) -> bool {
        match &self.form {
            Form::Long { name } => word
                .strip_prefix("--")
                .map(|given| given.split_once('=').map_or(given, |(option, _)| option))
                .is_some_and(|option| !option.is_empty() && name.starts_with(option)),
            Form::Short { letter } => word
                .strip_prefix('-')
                .is_some_and(|cluster| !cluster.starts_with('-') && cluster.contains(*letter)),
            Form::Word { name } => word
                .strip_prefix('-')
                .is_some_and(|given| given.starts_with(name.as_str())),
        }
    }
}
