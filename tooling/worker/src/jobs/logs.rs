use super::storage;
use anyhow::Result;
use serde_json::{Value, json};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

pub fn read(runtime: &Path, id: &str) -> Result<Value> {
    storage::load(runtime, id)?;
    let directory = runtime.join("jobs").join(id);
    Ok(
        json!({"run_id": id, "stdout": tail(&directory.join("stdout.log"))?,
        "stderr": tail(&directory.join("stderr.log"))?, "launcher": tail(&directory.join("launcher.log"))?}),
    )
}
fn tail(path: &Path) -> Result<Value> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(json!({"text": "", "bytes": 0, "truncated": false}));
        }
        Err(error) => return Err(error.into()),
    };
    let size = file.metadata()?.len();
    let offset = size.saturating_sub(65536);
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = Vec::new();
    file.take(65536).read_to_end(&mut bytes)?;
    Ok(json!({"text": String::from_utf8_lossy(&bytes), "bytes": size, "truncated": offset != 0}))
}
