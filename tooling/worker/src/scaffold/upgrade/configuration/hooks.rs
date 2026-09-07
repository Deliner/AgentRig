use super::{config, manifest, package};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

pub(super) fn file(
    root: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
    current: &config::Config,
    path: &str,
) -> Result<()> {
    let missing = !root.join(path).try_exists()?;
    if missing {
        return Ok(());
    }
    let mut existing = serde_json::from_slice(&fs::read(root.join(path))?)?;
    let desired = serde_json::from_slice(&files[path])?;
    merge(root, current, &mut existing, &desired)?;
    let bytes = serde_json::to_vec_pretty(&existing)?;
    let customized = bytes != files[path];
    if customized {
        // Merged user hooks are local adapter content, not a replacement stock template.
        let receipt_path = current.paths.service_path("manifest.json");
        let mut receipt: manifest::Manifest = serde_json::from_slice(&files[&receipt_path])?;
        receipt
            .local
            .insert(path.into(), Some(manifest::checksum(&bytes)));
        files.insert(receipt_path, serde_json::to_vec_pretty(&receipt)?);
    }
    files.insert(path.into(), bytes);
    Ok(())
}

pub(super) fn object<'a>(
    value: &'a mut Value,
    name: &str,
) -> Result<&'a mut serde_json::Map<String, Value>> {
    value
        .as_object_mut()
        .context("client settings must be an object")?
        .entry(name)
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .with_context(|| format!("{name} must be an object; existing settings preserved"))
}

pub(super) fn merge(
    root: &Path,
    current: &config::Config,
    existing: &mut Value,
    desired: &Value,
) -> Result<()> {
    let generated = package::adapters::generated(root, current)?;
    let previous: Value = serde_json::from_slice(&package::adapters::registration(
        current,
        &generated.root_command,
    )?)?;
    let old = previous["hooks"]
        .as_object()
        .context("generated hooks object required")?;
    let new = desired["hooks"]
        .as_object()
        .context("generated hooks object required")?;
    let hooks = object(existing, "hooks")?;
    let events: std::collections::BTreeSet<_> = old.keys().chain(new.keys()).collect();
    for event in events {
        let groups = hooks
            .entry(event)
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .with_context(|| {
                format!("hooks.{event} must be an array; existing settings preserved")
            })?;
        replace_groups(groups, old.get(event), new.get(event));
    }
    Ok(())
}

fn replace_groups(groups: &mut Vec<Value>, previous: Option<&Value>, desired: Option<&Value>) {
    let old = previous.and_then(Value::as_array);
    let new = desired.and_then(Value::as_array);
    groups.retain(|group| {
        old.is_none_or(|items| !items.contains(group))
            || new.is_some_and(|items| items.contains(group))
    });
    for group in new.into_iter().flatten() {
        let absent = !groups.contains(group);
        if absent {
            groups.push(group.clone());
        }
    }
}
