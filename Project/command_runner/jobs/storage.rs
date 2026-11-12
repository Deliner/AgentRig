use super::Record;
use crate::shared::fs;
use anyhow::{Result, ensure};
use std::path::{Path, PathBuf};

pub fn create(runtime: &Path) -> Result<PathBuf> {
    let root = runtime.join("jobs");
    fs::create_dir_all(&root)?;
    // DECISION: D092
    Ok(tempfile::Builder::new()
        .prefix("run-")
        .tempdir_in(root)?
        .keep())
}
pub fn save(directory: &Path, record: &Record) -> Result<()> {
    crate::shared::artifacts::json::save(&directory.join("record.json"), record)
}
pub fn load(runtime: &Path, id: &str) -> Result<Record> {
    ensure!(
        !id.is_empty()
            && id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte)),
        "invalid run ID"
    );
    let path = runtime.join("jobs").join(id).join("record.json");
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
pub fn list(runtime: &Path) -> Result<Vec<Record>> {
    let Some(entries) = fs::present(fs::read_dir(runtime.join("jobs")))? else {
        return Ok(Vec::new());
    };
    let mut records = Vec::new();
    for entry in entries {
        let path = entry?.path().join("record.json");
        let Some(saved) = fs::present(fs::read(path))? else {
            continue;
        };
        records.push(serde_json::from_slice::<Record>(&saved)?);
    }
    records.sort_by(|a, b| (a.started, &a.run_id).cmp(&(b.started, &b.run_id)));
    Ok(records)
}
