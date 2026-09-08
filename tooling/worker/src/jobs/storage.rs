use super::Record;
use anyhow::{Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn create(runtime: &Path) -> Result<PathBuf> {
    let root = runtime.join("jobs");
    fs::create_dir_all(&root)?;
    Ok(tempfile::Builder::new()
        .prefix("run-")
        .tempdir_in(root)?
        .keep())
}
pub fn save(directory: &Path, record: &Record) -> Result<()> {
    review_runner::artifacts::json::save(&directory.join("record.json"), record)
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
    let directory = runtime.join("jobs");
    let missing = !directory.exists();
    if missing {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path().join("record.json");
        let pending_creation = !path.exists();
        if pending_creation {
            continue;
        }
        records.push(serde_json::from_slice::<Record>(&fs::read(path)?)?);
    }
    records.sort_by(|a, b| (a.started, &a.run_id).cmp(&(b.started, &b.run_id)));
    Ok(records)
}
