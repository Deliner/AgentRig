// DECISION: D160
use crate::project_config::Context;
use crate::shared::{
    fs,
    vcs::{FileKind, Repository},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path, time::SystemTime};

#[derive(Deserialize, Serialize)]
struct Stamp {
    object: String,
    modified: SystemTime,
}

pub(super) fn reuse(root: &Path, context: &mut Context) -> Result<fs::File> {
    let directory = context.runtime.join("staged");
    fs::create_dir_all(&directory)?;
    let lock = fs::lock(directory.join("lock"))?;
    let manifest = directory.join("stamps.json");
    let previous = recorded(&manifest)?;
    let now = SystemTime::now();
    let mut stamps = BTreeMap::new();
    for (path, entry) in Repository::new(root).index()? {
        let stamped = match entry.kind {
            FileKind::File | FileKind::Executable => true,
            FileKind::Symlink | FileKind::Submodule => false,
        };
        if stamped {
            let unchanged = previous
                .get(&path)
                .filter(|stamp| stamp.object == entry.object);
            let modified = unchanged.map_or(now, |stamp| stamp.modified);
            fs::set_modified(context.root.join(&path), modified)?;
            let object = entry.object;
            stamps.insert(path, Stamp { object, modified });
        }
    }
    fs::write(&manifest, serde_json::to_vec(&stamps)?)?;
    context.cargo_target = Some(directory.join("target"));
    Ok(lock)
}

fn recorded(manifest: &Path) -> Result<BTreeMap<String, Stamp>> {
    match fs::read(manifest) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes).unwrap_or_default()),
        Err(error) if error.missing() => Ok(BTreeMap::new()),
        Err(error) => Err(error.into()),
    }
}
