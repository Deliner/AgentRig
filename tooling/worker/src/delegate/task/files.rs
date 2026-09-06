use super::{Profile, Request};
use anyhow::{Result, ensure};
use review_runner::{config::globs, digest, response::read_regular, snapshot::safe_path};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path},
};

pub(super) fn relative(path: &str) -> Result<()> {
    ensure!(
        !path.is_empty()
            && Path::new(path)
                .components()
                .all(|component| matches!(component, Component::Normal(_))),
        "task path must be relative without traversal: {path}"
    );
    safe_path(path)
}

pub(super) fn copy_inputs(
    paths: (&Path, &Path),
    request: &Request,
    profile: &Profile,
    manifest: &mut BTreeMap<String, String>,
) -> Result<()> {
    let (root, target) = paths;
    let allowed = globs(&profile.visible_paths)?;
    for (name, source) in &request.inputs {
        ensure!(
            allowed.is_match(source),
            "input outside visible_paths: {source}"
        );
        let path = super::path(root, source)?;
        let bytes = read_regular(&path, fs::metadata(&path)?.len())?;
        let output = target.join(name);
        fs::create_dir_all(output.parent().unwrap())?;
        fs::write(output, &bytes)?;
        manifest.insert(format!("inputs/{name}"), digest(&bytes));
    }
    Ok(())
}
