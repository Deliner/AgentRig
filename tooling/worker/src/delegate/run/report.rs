use crate::{delegate::task, util::save_json};
use anyhow::{Context, Result, ensure};
use review_runner::response::read_regular;
use serde_json::{Value, json};
use std::{fs, path::Path};

pub(super) fn artifacts(directory: &Path, contract: &task::Contract) -> Result<()> {
    let target = directory.join("artifacts");
    fs::create_dir(&target)?;
    for (name, limit) in &contract.artifacts {
        let source = task::path(&directory.join("private/work"), name)?;
        let output = target.join(name);
        fs::create_dir_all(output.parent().context("artifact needs parent")?)?;
        fs::write(output, read_regular(&source, *limit)?)?;
    }
    Ok(())
}

pub(super) fn finish(directory: &Path, result: Result<Value>) -> Result<bool> {
    let mut report = match result {
        Ok(result) => json!({"status":"PASS","result":result}),
        Err(error) => json!({"status":"ERROR","error":format!("{error:#}")}),
    };
    save_json(&directory.join("delegate-result.json"), &report)?;
    cleanup(directory, &mut report)
}

fn cleanup(directory: &Path, report: &mut Value) -> Result<bool> {
    let errors: Vec<_> = ["input", "private"]
        .iter()
        .filter_map(|name| {
            let path = directory.join(name);
            remove(&path).err().map(|error| format!("{name}: {error}"))
        })
        .collect();
    let success = report["status"] == "PASS" && errors.is_empty();
    report["cleanup_errors"] = json!(errors);
    save_json(&directory.join("delegate-result.json"), report)?;
    Ok(success)
}

fn remove(path: &Path) -> Result<()> {
    let exists = path.try_exists()?;
    if exists {
        fs::remove_dir_all(path)?;
    }
    ensure!(!path.try_exists()?, "temporary path remains after cleanup");
    Ok(())
}

pub(super) fn recover(directory: &Path, state: &Value) -> Result<()> {
    use fs2::FileExt;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(directory.join("report.lock"))?;
    lock.lock_exclusive()?;
    let path = directory.join("delegate-result.json");
    let mut report: Value = match fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            json!({"status":"ERROR","error":format!("delegate stopped without a result: {state}")})
        }
        Err(error) => return Err(error.into()),
    };
    let complete = report["cleanup_errors"]
        .as_array()
        .is_some_and(Vec::is_empty);
    let needs_cleanup = !complete;
    if needs_cleanup {
        save_json(&path, &report)?;
        cleanup(directory, &mut report)?;
    }
    Ok(())
}
