use crate::shared::fs_with_context as fs;
use anyhow::{Context, Result};
use globset::Glob;
use serde::{Deserialize, de::IgnoredAny};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

pub struct Layout {
    pub manifest: String,
    pub roots: Vec<String>,
    pub crates: BTreeMap<String, String>,
    // DECISION: D136
    pub externals: BTreeSet<String>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Manifest {
    package: Option<Package>,
    lib: Target,
    bin: Vec<Target>,
    example: Vec<Target>,
    test: Vec<Target>,
    bench: Vec<Target>,
    workspace: Workspace,
    #[serde(flatten)]
    dependencies: Dependencies,
    target: BTreeMap<String, Dependencies>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    build: Option<toml::Value>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Dependencies {
    dependencies: BTreeMap<String, IgnoredAny>,
    #[serde(rename = "dev-dependencies")]
    development: BTreeMap<String, IgnoredAny>,
    #[serde(rename = "build-dependencies")]
    build: BTreeMap<String, IgnoredAny>,
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
    pub fn fallback(source: &str) -> Self {
        Self {
            manifest: format!("{source}/Cargo.toml"),
            roots: vec![format!("{source}/src/lib.rs")],
            crates: BTreeMap::new(),
            externals: BTreeSet::new(),
        }
    }
}

pub fn layout(root: &Path, source: &str) -> Result<Layout> {
    let found = [format!("{source}/Cargo.toml"), "Cargo.toml".to_owned()]
        .into_iter()
        .find(|candidate| root.join(candidate).is_file());
    let Some(manifest) = found else {
        return Ok(Layout::fallback(source));
    };
    manifest_layout(root, &manifest)
}

// DECISION: D144
pub fn manifest_layout(root: &Path, manifest: &str) -> Result<Layout> {
    let directory = Path::new(manifest).parent().unwrap_or(Path::new(""));
    let parsed = read(&root.join(manifest))?;
    let mut layout = Layout {
        manifest: manifest.to_owned(),
        roots: Vec::new(),
        crates: BTreeMap::new(),
        externals: BTreeSet::new(),
    };
    let workspace = PackageDirectory { root, directory };
    workspace.add(&parsed, &mut layout);
    for member in workspace.members(&parsed.workspace)? {
        let path = root.join(&member).join("Cargo.toml");
        let package = path.is_file();
        if package {
            let directory = &member;
            PackageDirectory { root, directory }.add(&read(&path)?, &mut layout);
        }
    }
    Ok(layout)
}

fn read(path: &Path) -> Result<Manifest> {
    let text = fs::read_to_string(path)?;
    toml::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

#[derive(Clone, Copy)]
struct PackageDirectory<'a> {
    root: &'a Path,
    directory: &'a Path,
}

impl PackageDirectory<'_> {
    fn add(self, manifest: &Manifest, layout: &mut Layout) {
        let Some(package) = &manifest.package else {
            return;
        };
        let library = manifest.lib.path.as_deref().unwrap_or("src/lib.rs");
        let library = self.existing(library);
        if let Some(path) = &library {
            layout
                .crates
                .insert(package.name.replace('-', "_"), path.clone());
        }
        // DECISION: D136
        let build = match &package.build {
            None | Some(toml::Value::Boolean(true)) => Some("build.rs"),
            Some(toml::Value::String(path)) => Some(path.as_str()),
            Some(_) => None,
        };
        let binaries = [
            &manifest.bin,
            &manifest.example,
            &manifest.test,
            &manifest.bench,
        ]
        .into_iter()
        .flatten()
        .filter_map(|target| target.path.as_deref())
        .chain(["src/main.rs"])
        .chain(build)
        .filter_map(|path| self.existing(path));
        for path in library.into_iter().chain(binaries) {
            let fresh = !layout.roots.contains(&path);
            if fresh {
                layout.roots.push(path);
            }
        }
        layout.externals.extend(dependency_names(manifest));
    }

    fn existing(self, path: &str) -> Option<String> {
        let mut relative = PathBuf::new();
        for component in self.directory.join(path).components() {
            match component {
                Component::Normal(part) => relative.push(part),
                Component::CurDir => {}
                Component::ParentDir => {
                    let escaped = !relative.pop();
                    if escaped {
                        return None;
                    }
                }
                Component::Prefix(_) | Component::RootDir => return None,
            }
        }
        let file = self.root.join(&relative).is_file();
        file.then(|| relative.to_string_lossy().into_owned())
    }

    fn members(self, workspace: &Workspace) -> Result<Vec<PathBuf>> {
        let mut found = Vec::new();
        for pattern in &workspace.members {
            let mut paths = vec![self.directory.to_path_buf()];
            for segment in pattern.split('/') {
                paths = expand(self.root, &paths, segment)?;
            }
            found.extend(paths);
        }
        let base = self.root.join(self.directory);
        found.retain(|member| {
            let path = self.root.join(member);
            let excluded = workspace
                .exclude
                .iter()
                .any(|skip| path.starts_with(base.join(skip)));
            !excluded
        });
        Ok(found)
    }
}

// DECISION: D136
fn dependency_names(manifest: &Manifest) -> impl Iterator<Item = String> + '_ {
    std::iter::once(&manifest.dependencies)
        .chain(manifest.target.values())
        .flat_map(|table| {
            let development = table.development.keys();
            table
                .dependencies
                .keys()
                .chain(development)
                .chain(table.build.keys())
        })
        .map(|name| name.replace('-', "_"))
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
        let Some(entries) = fs::present(fs::read_dir(&directory))? else {
            continue;
        };
        for entry in entries {
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
