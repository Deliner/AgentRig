use crate::shared::fs;
use anyhow::{Context, Result};
use globset::Glob;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

pub(super) struct Layout {
    pub manifest: String,
    pub roots: Vec<String>,
    pub crates: BTreeMap<String, String>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Manifest {
    package: Option<Package>,
    lib: Target,
    bin: Vec<Target>,
    workspace: Workspace,
}

#[derive(Deserialize)]
struct Package {
    name: String,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Target {
    path: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Workspace {
    members: Vec<String>,
    exclude: Vec<String>,
}

impl Layout {
    pub(super) fn fallback(source: &str) -> Self {
        Self {
            manifest: format!("{source}/Cargo.toml"),
            roots: vec![format!("{source}/src/lib.rs")],
            crates: BTreeMap::new(),
        }
    }
}

pub(super) fn layout(root: &Path, source: &str) -> Result<Layout> {
    let found = [format!("{source}/Cargo.toml"), "Cargo.toml".to_owned()]
        .into_iter()
        .find(|candidate| root.join(candidate).is_file());
    let Some(manifest) = found else {
        return Ok(Layout::fallback(source));
    };
    let directory = Path::new(&manifest).parent().unwrap_or(Path::new(""));
    let parsed = read(&root.join(&manifest))?;
    let mut layout = Layout {
        manifest: manifest.clone(),
        roots: Vec::new(),
        crates: BTreeMap::new(),
    };
    add_package(root, directory, &parsed, &mut layout);
    for member in members(root, directory, &parsed.workspace)? {
        let path = root.join(&member).join("Cargo.toml");
        let package = path.is_file();
        if package {
            add_package(root, &member, &read(&path)?, &mut layout);
        }
    }
    Ok(layout)
}

fn read(path: &Path) -> Result<Manifest> {
    let text = fs::read_to_string(path)?;
    toml::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

fn add_package(root: &Path, directory: &Path, manifest: &Manifest, layout: &mut Layout) {
    let Some(package) = &manifest.package else {
        return;
    };
    let library = manifest.lib.path.as_deref().unwrap_or("src/lib.rs");
    let library = existing(root, directory, library);
    if let Some(path) = &library {
        layout
            .crates
            .insert(package.name.replace('-', "_"), path.clone());
    }
    let binaries = manifest
        .bin
        .iter()
        .filter_map(|bin| bin.path.as_deref())
        .chain(["src/main.rs"])
        .filter_map(|path| existing(root, directory, path));
    for path in library.into_iter().chain(binaries) {
        let fresh = !layout.roots.contains(&path);
        if fresh {
            layout.roots.push(path);
        }
    }
}

fn existing(root: &Path, directory: &Path, path: &str) -> Option<String> {
    let mut relative = PathBuf::new();
    for component in directory.join(path).components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                let escaped = !relative.pop();
                if escaped {
                    return None;
                }
            }
            _ => return None,
        }
    }
    let file = root.join(&relative).is_file();
    file.then(|| relative.to_string_lossy().into_owned())
}

fn members(root: &Path, directory: &Path, workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for pattern in &workspace.members {
        let mut paths = vec![directory.to_path_buf()];
        for segment in pattern.split('/') {
            paths = expand(root, &paths, segment)?;
        }
        found.extend(paths);
    }
    let base = root.join(directory);
    found.retain(|member| {
        let path = root.join(member);
        let excluded = workspace
            .exclude
            .iter()
            .any(|skip| path.starts_with(base.join(skip)));
        !excluded
    });
    Ok(found)
}

fn expand(root: &Path, paths: &[PathBuf], segment: &str) -> Result<Vec<PathBuf>> {
    let recursive = segment == "**";
    if recursive {
        return descendants(root, paths);
    }
    let literal = !segment.contains(['*', '?', '[']);
    if literal {
        return Ok(paths.iter().map(|path| path.join(segment)).collect());
    }
    let matcher = Glob::new(segment)?.compile_matcher();
    let mut expanded = Vec::new();
    for path in paths {
        let directory = root.join(path);
        let missing = !directory.is_dir();
        if missing {
            continue;
        }
        for entry in fs::read_dir(&directory)? {
            let name = entry?.file_name();
            let selected = matcher.is_match(&name) && directory.join(&name).is_dir();
            if selected {
                expanded.push(path.join(name));
            }
        }
    }
    expanded.sort();
    Ok(expanded)
}

fn descendants(root: &Path, paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let mut level = paths.to_vec();
    while !level.is_empty() {
        let children = expand(root, &level, "*")?;
        found.append(&mut level);
        level = children;
    }
    found.sort();
    Ok(found)
}
