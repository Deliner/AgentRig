// DECISION: D026
use anyhow::{Context, Result, ensure};
use serde::{Serialize, de::DeserializeOwned};
use serde_yaml_ng::Value;
use std::{fs, path::Path};
use yaml_rust2::scanner::{Scanner, Token, TokenType};

pub fn read<T: DeserializeOwned>(path: &Path) -> Result<T> {
    read_document(path).map(|(value, _)| value)
}

pub fn read_document<T: DeserializeOwned>(path: &Path) -> Result<(T, String)> {
    let legacy = path
        .extension()
        .is_some_and(|extension| extension == "toml");
    ensure!(
        !legacy,
        "YAML configuration required: explicitly migrate legacy TOML {} to YAML; no format fallback is supported",
        path.display()
    );
    let source = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value =
        decode(&source).with_context(|| format!("YAML configuration {}", path.display()))?;
    Ok((value, source))
}

pub fn decode<T: DeserializeOwned>(source: &str) -> Result<T> {
    syntax(source)?;
    // Decode to a value first: mappings reject duplicate keys, and typed scalar
    // values prevent string coercion during the subsequent schema check.
    let value: Value = serde_yaml_ng::from_str(source)?;
    values(&value)?;
    let value = serde_json::to_value(value)?;
    Ok(serde_path_to_error::deserialize(value)?)
}

pub fn encode<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_yaml_ng::to_string(value)?)
}

fn syntax(source: &str) -> Result<()> {
    let mut scanner = Scanner::new(source.chars());
    for Token(location, token) in scanner.by_ref() {
        let unsupported = matches!(
            token,
            TokenType::Alias(_)
                | TokenType::Anchor(_)
                | TokenType::Tag(_, _)
                | TokenType::TagDirective(_, _)
        );
        ensure!(
            !unsupported,
            "YAML anchors, aliases and tags are unsupported at line {}, column {}; use explicit configuration packages",
            location.line(),
            location.col() + 1
        );
    }
    if let Some(error) = scanner.get_error() {
        return Err(error.into());
    }
    Ok(())
}

fn values(root: &Value) -> Result<()> {
    let mut pending = vec![root];
    while let Some(value) = pending.pop() {
        match value {
            Value::Mapping(mapping) => {
                for (key, child) in mapping {
                    let key = key
                        .as_str()
                        .context("YAML configuration keys must be strings")?;
                    ensure!(
                        key != "<<",
                        "YAML merge keys are unsupported; use explicit overrides"
                    );
                    pending.push(child);
                }
            }
            Value::Sequence(items) => pending.extend(items),
            Value::Number(number) => ensure!(
                number.as_f64().is_some_and(f64::is_finite),
                "YAML configuration numbers must be finite"
            ),
            _ => {}
        }
    }
    Ok(())
}
