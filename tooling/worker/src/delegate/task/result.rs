use super::{Contract, path};
use anyhow::Result;
use review_runner::{
    digest,
    response::{MAX_BYTES, read_regular},
};
use serde_json::{Value, json};
use std::path::Path;

pub fn verify(directory: &Path, contract: &Contract) -> Result<Value> {
    let response = path(directory, "result.json")?;
    let value: Value = serde_json::from_slice(&read_regular(&response, MAX_BYTES)?)?;
    super::validator(contract)?
        .validate(&value)
        .map_err(|error| anyhow::anyhow!("result schema: {error}"))?;
    let mut artifacts = serde_json::Map::new();
    for (name, limit) in &contract.artifacts {
        let file = path(directory, name)?;
        let bytes = read_regular(&file, *limit)?;
        artifacts.insert(
            name.clone(),
            json!({"sha256":digest(&bytes),"bytes":bytes.len()}),
        );
    }
    Ok(json!({"response":value,"artifacts":artifacts}))
}
