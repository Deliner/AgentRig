use anyhow::{Result, bail};
use serde_json::Value;
use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

// DECISION: D015
pub fn object(path: &Path) -> Value {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| serde_json::json!({}))
}
pub fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}
pub use review_runner::artifacts::json::save as save_json;
pub fn resolve(path: &Path) -> Result<PathBuf> {
    resolve_limited(path, 0)
}
fn resolve_limited(path: &Path, depth: u8) -> Result<PathBuf> {
    let cycle = depth > 32;
    if cycle {
        bail!("symlink cycle");
    }
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            other => result.push(other.as_os_str()),
        }
        let symlink = fs::symlink_metadata(&result).is_ok_and(|meta| meta.file_type().is_symlink());
        if symlink {
            let target = fs::read_link(&result)?;
            result.pop();
            result = resolve_limited(&result.join(target), depth + 1)?;
        }
    }
    Ok(result)
}
pub fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).current_dir(root).output()?;
    let failed = !output.status.success();
    if failed {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8(output.stdout)?.trim().into())
}

pub fn take_option(args: &mut Vec<String>, name: &str) -> Result<Option<String>> {
    let Some(index) = args
        .iter()
        .take_while(|arg| arg.as_str() != "--")
        .position(|arg| arg == name)
    else {
        return Ok(None);
    };
    args.remove(index);
    let has_value = index < args.len() && !args[index].starts_with("--");
    if has_value {
        Ok(Some(args.remove(index)))
    } else {
        bail!("{name} requires a value")
    }
}
