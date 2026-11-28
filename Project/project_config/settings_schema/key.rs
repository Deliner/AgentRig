use anyhow::{Error, Result, ensure};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

pub struct Key {
    letters: String,
}
impl TryFrom<String> for Key {
    type Error = Error;
    fn try_from(letters: String) -> Result<Self> {
        let sized = (2..=10).contains(&letters.len());
        let valid = sized && letters.bytes().all(|letter| letter.is_ascii_uppercase());
        ensure!(
            valid,
            "{letters:?} is not 2 to 10 capital Latin letters, such as RIG"
        );
        Ok(Self { letters })
    }
}
impl<'de> Deserialize<'de> for Key {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let letters = String::deserialize(deserializer)?;
        Self::try_from(letters).map_err(D::Error::custom)
    }
}
impl Serialize for Key {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.letters)
    }
}
