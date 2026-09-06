use super::{Layout, Profile};
use crate::resources::{Bundle, Input, digest};
use anyhow::Result;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, os::unix::fs::PermissionsExt};

pub(super) fn prepare(layout: &Layout, profile: &Profile) -> Result<Value> {
    let mut bundle = Bundle::new("environment");
    for (name, source) in &profile.environment.programs {
        bundle.copy_at(source, &format!("programs/{name}"))?;
    }
    for source in &profile.environment.skills {
        let name = source.file_name().unwrap().to_string_lossy();
        bundle.directory_at(source, &format!("skills/{name}"))?;
    }
    bundle.verify()?;
    let root = layout.private.join("environment");
    fs::create_dir(&root)?;
    let mut files = BTreeMap::new();
    for (name, file) in &bundle.files {
        let path = root.join(name);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, &file.bytes)?;
        let mode = if file.executable { 0o755 } else { 0o644 };
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
        files.insert(
            name,
            Input {
                sha256: digest(&file.bytes),
                executable: file.executable,
            },
        );
    }
    Ok(json!({"schema_version":1,"files":files,"inputs":bundle.inputs}))
}
