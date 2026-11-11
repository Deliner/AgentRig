// DECISION: D160
// DECISION: D161
use crate::project_config::Context;
use crate::shared::{
    fs,
    vcs::{Entry, FileKind, Repository},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::SystemTime,
};
use tempfile::TempDir;

#[derive(Default, Deserialize, Serialize)]
struct Stamps {
    tree: PathBuf,
    files: BTreeMap<String, Stamp>,
}

#[derive(Deserialize, Serialize)]
struct Stamp {
    object: String,
    modified: SystemTime,
}

pub(super) struct Staged {
    _tree: TempDir,
    _lock: fs::File,
}

pub(super) fn reuse(root: &Path, export: TempDir, context: &mut Context) -> Result<Staged> {
    let directory = context.runtime.join("staged");
    fs::create_dir_all(&directory)?;
    let lock = fs::lock(directory.join("lock"))?;
    let manifest = directory.join("stamps.json");
    let previous = recorded(&manifest)?;
    let tree = placed(&previous.tree, export)?;
    let runtime = context.runtime.clone();
    *context = Context {
        runtime,
        ..Context::load(tree.path())?
    };
    let same_tree = context.root == previous.tree;
    let earlier = if same_tree {
        previous.files
    } else {
        BTreeMap::new()
    };
    let index = Repository::new(root).index()?;
    let files = stamped(index, &context.root, &earlier)?;
    let tree_path = context.root.clone();
    let stamps = Stamps {
        tree: tree_path,
        files,
    };
    fs::write(&manifest, serde_json::to_vec(&stamps)?)?;
    context.cargo_target = Some(directory.join("target"));
    Ok(Staged {
        _tree: tree,
        _lock: lock,
    })
}

fn stamped(
    index: BTreeMap<String, Entry>,
    tree: &Path,
    earlier: &BTreeMap<String, Stamp>,
) -> Result<BTreeMap<String, Stamp>> {
    let now = SystemTime::now();
    let mut files = BTreeMap::new();
    for (path, entry) in index {
        let stamped = match entry.kind {
            FileKind::File | FileKind::Executable => true,
            FileKind::Symlink | FileKind::Submodule => false,
        };
        if stamped {
            let unchanged = earlier
                .get(&path)
                .filter(|stamp| stamp.object == entry.object);
            let modified = unchanged.map_or(now, |stamp| stamp.modified);
            fs::set_modified(tree.join(&path), modified)?;
            let object = entry.object;
            files.insert(path, Stamp { object, modified });
        }
    }
    Ok(files)
}

fn recorded(manifest: &Path) -> Result<Stamps> {
    match fs::read(manifest) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes).unwrap_or_default()),
        Err(error) if error.missing() => Ok(Stamps::default()),
        Err(error) => Err(error.into()),
    }
}

fn placed(recorded: &Path, export: TempDir) -> Result<TempDir> {
    let named = recorded.file_name().and_then(|name| {
        let mut builder = tempfile::Builder::new();
        builder.prefix(name).rand_bytes(0).tempdir().ok()
    });
    let tree = match named {
        Some(tree) => tree,
        None => tempfile::Builder::new()
            .prefix("agentrig-staged-")
            .tempdir()?,
    };
    let from = export.path();
    let to = tree.path();
    fs::rename(fs::Move { from, to })?;
    // DECISION: D161
    let _moved = export.keep();
    Ok(tree)
}
